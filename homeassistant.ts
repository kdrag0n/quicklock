import { HA_API_KEY, HA_ENTITY } from './config'
import fetch from 'node-fetch'

export async function postLock(unlocked: boolean) {
  const service = unlocked ? 'unlock' : 'lock'

  await fetch(`http://171.66.3.236:8123/api/services/lock/${service}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${HA_API_KEY}`,
    },
    body: JSON.stringify({ entity_id: HA_ENTITY }),
  })
}
