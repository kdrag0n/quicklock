package dev.kdrag0n.quicklock.server

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.single
import kotlinx.coroutines.flow.take
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NfcApiServiceWrapper @Inject constructor() : ApiService {
    val serviceFlow = MutableStateFlow<NfcApiService?>(null)

    override suspend fun getEntities() = useTag {
        Timber.d("getEntities")
        it.getEntities()
    }

    override suspend fun startInitialPair() = useTag {
        Timber.d("startInitialPair")
        it.startInitialPair()
    }

    override suspend fun finishInitialPair(request: InitialPairFinishRequest) = useTag {
        Timber.d("finishInitialPair")
        it.finishInitialPair(request)
    }

    override suspend fun getDelegatedPairFinishPayload(challengeId: String) = useTag {
        Timber.d("getDelegatedPairFinishPayload")
        it.getDelegatedPairFinishPayload(challengeId)
    }

    override suspend fun uploadDelegatedPairFinishPayload(
        challengeId: String,
        payload: PairFinishPayload
    ) = useTag {
        Timber.d("uploadDelegatedPairFinishPayload")
        it.uploadDelegatedPairFinishPayload(challengeId, payload)
    }

    override suspend fun finishDelegatedPair(
        challengeId: String,
        request: SignedRequestEnvelope<Delegation>
    ) = useTag {
        Timber.d("finishDelegatedPair")
        it.finishDelegatedPair(challengeId, request)
    }

    override suspend fun getPairingChallenge() = useTag {
        Timber.d("getPairingChallenge")
        it.getPairingChallenge()
    }

    override suspend fun startUnlock(request: UnlockStartRequest) = useTag {
        Timber.d("startUnlock")
        it.startUnlock(request)
    }

    override suspend fun finishUnlock(
        challengeId: String,
        request: SignedRequestEnvelope<UnlockChallenge>
    ) = useTag {
        Timber.d("finishUnlock")
        it.finishUnlock(challengeId, request)
    }

    private suspend fun <T> useTag(block: suspend (ApiService) -> T): T {
        Timber.d("useTag")

        val service = serviceFlow
            .filterNotNull()
            .take(1)
            .single()

        if (!service.test()) {
            Timber.d("NFC disconnected, waiting again")
            serviceFlow.value = null
            return useTag(block)
        }

        return block(service)
    }
}
