import fastify from 'fastify'
import crypto from 'crypto'
import fsSync from 'fs'
import fs from 'fs/promises'
import { HA_API_KEY, HA_ENTITY, PAIRING_KEY } from './config'
import fetch from 'node-fetch'

const server = fastify({ logger: true })

function cryptoCompare(a: string, b: string) {
  return crypto.timingSafeEqual(Buffer.from(a), Buffer.from(b))
}

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
  keys.push(key)
  await fs.writeFile('data/keys.json', JSON.stringify(keys))
}

interface PairRequest {
  publicKey: string
  pairingKey: string
}
server.post('/api/pair', async (request, reply) => {
  const { publicKey, pairingKey } = request.body as PairRequest
  if (!cryptoCompare(pairingKey, PAIRING_KEY)) {
    reply.code(401).send('Invalid pairing key')
    return
  }

  await addKey(publicKey)
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
    console.log('Invalid signature')
    reply.code(401).send('Invalid signature')
    return
  }

  // Verify timestamp: 2 min
  if (Math.abs(Date.now() - timestamp) > 2 * 60 * 1000) {
    console.log('Invalid timestamp')
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
