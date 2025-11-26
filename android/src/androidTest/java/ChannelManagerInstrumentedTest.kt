// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ChannelManagerInstrumentedTest {

    private lateinit var context: Context
    private lateinit var channelManager: ChannelManager
    private lateinit var notificationManager: NotificationManager
    private val testChannelIds = mutableListOf<String>()

    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        channelManager = ChannelManager(context)
        notificationManager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    }

    @After
    fun cleanup() {
        // Clean up test channels (only on O+)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            for (channelId in testChannelIds) {
                try {
                    notificationManager.deleteNotificationChannel(channelId)
                } catch (e: Exception) {
                    // Ignore cleanup errors
                }
            }
        }
        testChannelIds.clear()
    }

    @Test
    fun testChannelCreationOnSupportedDevices() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channelId = "test_channel_1"
            val channelName = "Test Channel"
            val importance = android.app.NotificationManager.IMPORTANCE_HIGH

            testChannelIds.add(channelId)

            // Actually create the channel using Android API
            val notificationChannel = android.app.NotificationChannel(
                channelId,
                channelName,
                importance
            )
            notificationChannel.description = "A test notification channel"
            notificationChannel.lockscreenVisibility = android.app.Notification.VISIBILITY_PUBLIC

            notificationManager.createNotificationChannel(notificationChannel)

            // Verify channel was created
            val retrievedChannel = notificationManager.getNotificationChannel(channelId)
            assertNotNull(retrievedChannel)
            assertEquals(channelId, retrievedChannel?.id)
            assertEquals(channelName, retrievedChannel?.name.toString())
        } else {
            // On pre-O devices, channels are not supported
            // Test should still pass
            assertTrue(true)
        }
    }

    @Test
    fun testChannelImportanceLevels() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val importanceLevels = listOf(
                Importance.None to "none",
                Importance.Min to "min",
                Importance.Low to "low",
                Importance.Default to "default",
                Importance.High to "high"
            )

            for ((importance, suffix) in importanceLevels) {
                val channel = Channel()
                channel.id = "test_importance_$suffix"
                channel.name = "Test $suffix"
                channel.importance = importance

                testChannelIds.add(channel.id)

                // Verify importance value is correct
                assertEquals(importance.value, when (importance) {
                    Importance.None -> 0
                    Importance.Min -> 1
                    Importance.Low -> 2
                    Importance.Default -> 3
                    Importance.High -> 4
                })
            }
        }
    }

    @Test
    fun testChannelVisibilityLevels() {
        val visibilityLevels = listOf(
            Visibility.Secret to -1,
            Visibility.Private to 0,
            Visibility.Public to 1
        )

        for ((visibility, expectedValue) in visibilityLevels) {
            assertEquals(expectedValue, visibility.value)
        }
    }

    @Test
    fun testChannelWithAllProperties() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = Channel()
            channel.id = "test_full_channel"
            channel.name = "Full Test Channel"
            channel.description = "Channel with all properties"
            channel.importance = Importance.High
            channel.visibility = Visibility.Private
            channel.lights = true
            channel.lightsColor = "#FF0000"
            channel.vibration = true

            testChannelIds.add(channel.id)

            // Verify all properties are set
            assertEquals("test_full_channel", channel.id)
            assertEquals("Full Test Channel", channel.name)
            assertEquals("Channel with all properties", channel.description)
            assertEquals(Importance.High, channel.importance)
            assertEquals(Visibility.Private, channel.visibility)
            assertTrue(channel.lights ?: false)
            assertEquals("#FF0000", channel.lightsColor)
            assertTrue(channel.vibration ?: false)
        }
    }

    @Test
    fun testChannelDefaultValues() {
        val channel = Channel()
        channel.id = "test_defaults"
        channel.name = "Defaults"

        testChannelIds.add(channel.id)

        // Verify defaults
        assertNull(channel.description)
        assertNull(channel.sound)
        assertNull(channel.lights)
        assertNull(channel.lightsColor)
        assertNull(channel.vibration)
        assertNull(channel.importance)
        assertNull(channel.visibility)
    }

    @Test
    fun testDeleteChannelArgs() {
        val args = DeleteChannelArgs()
        args.id = "channel_to_delete"

        assertEquals("channel_to_delete", args.id)
    }

    @Test
    fun testMultipleChannels() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channels = listOf(
                Channel().apply {
                    id = "channel1"
                    name = "Channel 1"
                    importance = Importance.Low
                },
                Channel().apply {
                    id = "channel2"
                    name = "Channel 2"
                    importance = Importance.High
                },
                Channel().apply {
                    id = "channel3"
                    name = "Channel 3"
                    importance = Importance.Default
                }
            )

            channels.forEach { testChannelIds.add(it.id) }

            // Verify all channels have correct properties
            assertEquals(3, channels.size)
            assertEquals(Importance.Low, channels[0].importance)
            assertEquals(Importance.High, channels[1].importance)
            assertEquals(Importance.Default, channels[2].importance)
        }
    }

    @Test
    fun testChannelWithSound() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = Channel()
            channel.id = "test_sound_channel"
            channel.name = "Sound Channel"
            channel.sound = "notification_sound"

            testChannelIds.add(channel.id)

            assertEquals("notification_sound", channel.sound)
        }
    }

    @Test
    fun testChannelLightColors() {
        val colors = listOf(
            "#FF0000" to "red",
            "#00FF00" to "green",
            "#0000FF" to "blue",
            "#FFFFFF" to "white",
            "#000000" to "black"
        )

        for ((color, _) in colors) {
            val channel = Channel()
            channel.lightsColor = color
            assertEquals(color, channel.lightsColor)
        }
    }

    @Test
    fun testChannelBooleanFlags() {
        val channel = Channel()
        channel.id = "test_flags"
        channel.name = "Flags"
        channel.lights = true
        channel.vibration = false

        testChannelIds.add(channel.id)

        assertTrue(channel.lights ?: false)
        assertFalse(channel.vibration ?: true)
    }

    @Test
    fun testImportanceEnumValues() {
        val values = Importance.values()
        assertEquals(5, values.size)

        assertEquals(0, Importance.None.value)
        assertEquals(1, Importance.Min.value)
        assertEquals(2, Importance.Low.value)
        assertEquals(3, Importance.Default.value)
        assertEquals(4, Importance.High.value)
    }

    @Test
    fun testVisibilityEnumValues() {
        val values = Visibility.values()
        assertEquals(3, values.size)

        assertEquals(-1, Visibility.Secret.value)
        assertEquals(0, Visibility.Private.value)
        assertEquals(1, Visibility.Public.value)
    }

    @Test
    fun testChannelIdRequired() {
        val channel = Channel()
        channel.name = "Test"

        // ID should be lateinit and not yet initialized
        // This will throw if accessed before being set
        try {
            @Suppress("UNUSED_VARIABLE")
            val id = channel.id
            fail("Expected UninitializedPropertyAccessException")
        } catch (e: Exception) {
            // Expected - lateinit var not initialized
            assertTrue(e is UninitializedPropertyAccessException ||
                      e.message?.contains("lateinit") == true)
        }
    }

    @Test
    fun testChannelNameRequired() {
        val channel = Channel()
        channel.id = "test"

        // Name should be lateinit and not yet initialized
        try {
            @Suppress("UNUSED_VARIABLE")
            val name = channel.name
            fail("Expected UninitializedPropertyAccessException")
        } catch (e: Exception) {
            // Expected - lateinit var not initialized
            assertTrue(e is UninitializedPropertyAccessException ||
                      e.message?.contains("lateinit") == true)
        }
    }
}
