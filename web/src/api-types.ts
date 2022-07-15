

export interface Entity {
  id: string
  name: string
  haEntity: string
}

export interface PairingChallenge {
  id: string
  timestamp: number
  isInitial: boolean
}

export interface PairFinishChallengeWA {
  pairChallengeId: string
}

export interface UnlockStartRequest {
  entityId: string
}

export interface PairFinishWA {
  keyId: string
  attestationObject: string
  clientDataJSON: string
}

export interface InitialPairFinishRequest {
  finishPayload: string
  mac: string
}

export interface UnlockFinishWA {
  keyId: string
  signature: string
  clientDataJSON: string
  authenticatorData: string
}

export interface Delegation {
  finishPayload: string
  expiresAt: number
  allowedEntities: string[] | null
}

export interface DelegatedPairFinishWA {
  delegationKeyId: string
  signature: string
  clientDataJSON: string
  authenticatorData: string
}
