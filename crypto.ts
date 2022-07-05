import { timingSafeEqual, X509Certificate } from 'crypto'

export function cryptoCompare(a: string, b: string) {
  return timingSafeEqual(Buffer.from(a), Buffer.from(b))
}

export function canonicalizeCert(data: string | Buffer) {
  const cert = new X509Certificate(data)
  return cert.toString()
}

export function verifyRootCert(cert: X509Certificate, roots: string[]) {
  const canonical = cert.toString()
  for (const root of roots) {
    if (cryptoCompare(root, canonical)) {
      return
    }
  }

  throw new Error('Invalid root certificate')
}

export function verifyCertChain(chain: string[], roots: string[]) {
  const certs = chain.map(encoded => new X509Certificate(Buffer.from(encoded, 'base64')))
  const now = Date.now()

  // Initial parent is root (last)
  const root = certs[certs.length - 1]
  let parent = root
  // Verify each cert starting from root, including parent
  certs.reverse().forEach(cert => {
    console.log(cert)

    // Time
    if (new Date(cert.validFrom).getTime() > now || new Date(cert.validTo).getTime() < now) {
      throw new Error('Certificate not valid (expired or not yet valid)')
    }

    // Signature
    if (!cert.checkIssued(parent)) {
      throw new Error('Certificate not issued by parent')
    }
    if (!cert.verify(parent.publicKey)) {
      throw new Error('Certificate signature invalid')
    }

    parent = cert
  })

  // Verify root
  verifyRootCert(root, roots)
}
