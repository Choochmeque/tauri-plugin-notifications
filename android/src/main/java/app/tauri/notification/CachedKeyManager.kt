package app.tauri.notification

import android.content.Context
import org.unifiedpush.android.connector.keys.DefaultKeyManager
import org.unifiedpush.android.connector.keys.KeyManager
import org.unifiedpush.android.connector.data.PublicKeySet
import java.util.concurrent.ConcurrentHashMap

/**
 * Wraps DefaultKeyManager with in-memory caching to avoid repeated
 * AndroidKeyStore access for getPublicKeySet/exists calls.
 */
class CachedKeyManager private constructor(context: Context) : KeyManager {
    private val delegate = DefaultKeyManager(context)
    private val pubkeyCache = ConcurrentHashMap<String, PublicKeySet>()
    private val existsCache = ConcurrentHashMap<String, Boolean>()

    override fun decrypt(instance: String, sealed: ByteArray): ByteArray? {
        return delegate.decrypt(instance, sealed)
    }

    override fun generate(instance: String) {
        pubkeyCache.remove(instance)
        existsCache.remove(instance)
        delegate.generate(instance)
    }

    override fun getPublicKeySet(instance: String): PublicKeySet? {
        pubkeyCache[instance]?.let { return it }
        val keys = delegate.getPublicKeySet(instance)
        if (keys != null) pubkeyCache[instance] = keys
        return keys
    }

    override fun exists(instance: String): Boolean {
        return existsCache.getOrPut(instance) { delegate.exists(instance) }
    }

    override fun delete(instance: String) {
        pubkeyCache.remove(instance)
        existsCache.remove(instance)
        delegate.delete(instance)
    }

    companion object {
        @Volatile
        private var instance: CachedKeyManager? = null

        fun getInstance(context: Context): CachedKeyManager {
            return instance ?: synchronized(this) {
                instance ?: CachedKeyManager(context.applicationContext).also { instance = it }
            }
        }
    }
}
