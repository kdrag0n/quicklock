import fastify from 'fastify'
import crypto, { verify, X509Certificate } from 'crypto'
import fsSync from 'fs'
import fs from 'fs/promises'
import { HA_API_KEY, HA_ENTITY, PAIRING_SECRET, TIME_GRACE_PERIOD } from './config'
import fetch from 'node-fetch'
import { cryptoCompare, verifyCertChain, verifyRootCert } from './crypto'
import { GOOGLE_ROOT_CERTS } from './certificates'

const server = fastify({ logger: true })

// Ongoing pairing challenges
interface Challenge {
  id: string
  timestamp: number
}
const challenges: Record<string, Challenge> = {}

// Simple storage of paired public keys in a JSON file
function loadKeys() {
  try {
    const keys = fsSync.readFileSync('data/keys.json', 'utf8')
    return JSON.parse(keys) as string[]
  } catch (e) {
    return []
  }
}

const keys = loadKeys()
async function addKey(key: string) {
  if (keys.includes(key)) {
    return
  }

  keys.push(key)
  await fs.writeFile('data/keys.json', JSON.stringify(keys))
}

interface PairStartRequest {
  pairingSecret: string
}
server.post('/api/pair/start', async (request, reply) => {
  const { pairingSecret } = request.body as PairStartRequest
  if (!cryptoCompare(pairingSecret, PAIRING_SECRET)) {
    reply.code(401).send('Invalid pairing secret')
    return
  }

  // Generate a new challenge
  const challengeId = crypto.randomBytes(32).toString('hex')
  const challenge = {
    id: challengeId,
    timestamp: Date.now(),
  }
  challenges[challengeId] = challenge

  return challenge
})

interface PairFinishRequest {
  challengeId: string
  publicKey: string
  attestationChain: string[]
}
server.post('/api/pair/finish', async (request, reply) => {
  const { publicKey, challengeId, attestationChain } = request.body as PairFinishRequest
  const challenge = challenges[challengeId]
  if (!challenge) {
    reply.code(400).send('Invalid challenge')
    return
  }

  try {
    // Verify timestamp
    const now = Date.now()
    if (now - challenge.timestamp > TIME_GRACE_PERIOD) {
      reply.code(400).send('Challenge expired')
      return
    }

    // Verify atttestation certificate chain
    verifyCertChain(attestationChain, GOOGLE_ROOT_CERTS)

    await addKey(publicKey)
  } catch (e) {
    reply.code(401).send('Invalid attestation chain')
    return
  } finally {
    // Drop challenge
    delete challenges[challengeId]
  }
})

interface UnlockPayload {
  publicKey: string
  timestamp: number
}
interface WrappedUnlockRequest {
  payload: string
  signature: string
}
server.post('/api/unlock', async (request, reply) => {
  const { payload, signature } = request.body as WrappedUnlockRequest
  const { publicKey, timestamp } = JSON.parse(payload) as UnlockPayload
  if (!keys.includes(publicKey)) {
    reply.code(401).send('Invalid key')
    return
  }

  console.log(request.body, JSON.parse(payload))

  const verifier = crypto.createVerify('sha256')
  verifier.update(payload)
  // Create PEM container for the base64-encoded DER.
  // This is in X.509 subjectPublicKeyInfo format.
  const valid = verifier.verify(`-----BEGIN PUBLIC KEY-----
${publicKey}
-----END PUBLIC KEY-----`, signature, 'base64')
  if (!valid) {
    reply.code(401).send('Invalid signature')
    return
  }

  // Verify timestamp: 2 min
  if (Math.abs(Date.now() - timestamp) > TIME_GRACE_PERIOD) {
    reply.code(401).send('Invalid timestamp')
    return
  }

  // Unlock
  console.log('Posting HA unlock')
  await fetch('http://171.66.3.236:8123/api/services/lock/unlock', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${HA_API_KEY}`,
    },
    body: JSON.stringify({ entity_id: HA_ENTITY }),
  })
})

async function start() {
  fsSync.mkdirSync('data', { recursive: true })
  await server.listen({ port: 3002, host: '0.0.0.0' })
}
start()
