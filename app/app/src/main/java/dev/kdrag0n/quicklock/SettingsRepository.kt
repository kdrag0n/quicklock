package dev.kdrag0n.quicklock

import android.content.Context
import com.chibatching.kotpref.KotprefModel
import dagger.Reusable
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject

@Reusable
class SettingsRepository @Inject constructor(
    @ApplicationContext context: Context,
) : KotprefModel(context) {
    var macKey by nullableStringPref()
    var encKey by nullableStringPref()
    var auditServerPublicKey by nullableStringPref()
    var auditClientId by nullableStringPref()
}
