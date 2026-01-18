package app.tauri.notification

import org.junit.Assert.*
import org.junit.Test

class ChannelManagerTest {

    @Test
    fun testImportance_values() {
        assertEquals(0, Importance.None.value)
        assertEquals(1, Importance.Min.value)
        assertEquals(2, Importance.Low.value)
        assertEquals(3, Importance.Default.value)
        assertEquals(4, Importance.High.value)
    }

    @Test
    fun testImportance_ordinal() {
        val values = Importance.values()
        assertEquals(5, values.size)
        assertEquals(Importance.None, values[0])
        assertEquals(Importance.Min, values[1])
        assertEquals(Importance.Low, values[2])
        assertEquals(Importance.Default, values[3])
        assertEquals(Importance.High, values[4])
    }

    @Test
    fun testVisibility_values() {
        assertEquals(-1, Visibility.Secret.value)
        assertEquals(0, Visibility.Private.value)
        assertEquals(1, Visibility.Public.value)
    }

    @Test
    fun testVisibility_ordinal() {
        val values = Visibility.values()
        assertEquals(3, values.size)
        assertEquals(Visibility.Secret, values[0])
        assertEquals(Visibility.Private, values[1])
        assertEquals(Visibility.Public, values[2])
    }

    @Test
    fun testChannel_properties() {
        val channel = Channel()
        channel.id = "test_channel"
        channel.name = "Test Channel"
        channel.description = "A test notification channel"
        channel.sound = "notification_sound"
        channel.lights = true
        channel.lightsColor = "#FF0000"
        channel.vibration = true
        channel.importance = Importance.High
        channel.visibility = Visibility.Private

        assertEquals("test_channel", channel.id)
        assertEquals("Test Channel", channel.name)
        assertEquals("A test notification channel", channel.description)
        assertEquals("notification_sound", channel.sound)
        assertEquals(true, channel.lights)
        assertEquals("#FF0000", channel.lightsColor)
        assertEquals(true, channel.vibration)
        assertEquals(Importance.High, channel.importance)
        assertEquals(Visibility.Private, channel.visibility)
    }

    @Test
    fun testChannel_nullableProperties() {
        val channel = Channel()
        channel.id = "minimal_channel"
        channel.name = "Minimal"

        assertEquals("minimal_channel", channel.id)
        assertEquals("Minimal", channel.name)
        assertNull(channel.description)
        assertNull(channel.sound)
        assertNull(channel.lights)
        assertNull(channel.lightsColor)
        assertNull(channel.vibration)
        assertNull(channel.importance)
        assertNull(channel.visibility)
    }

    @Test
    fun testChannel_defaultImportance() {
        val channel = Channel()
        channel.id = "default_channel"
        channel.name = "Default"

        val importance = channel.importance ?: Importance.Default
        assertEquals(Importance.Default, importance)
    }

    @Test
    fun testChannel_defaultVisibility() {
        val channel = Channel()
        channel.id = "default_channel"
        channel.name = "Default"

        val visibility = channel.visibility ?: Visibility.Private
        assertEquals(Visibility.Private, visibility)
    }

    @Test
    fun testChannel_allImportanceLevels() {
        val channels = listOf(
            Channel().apply {
                id = "none"
                name = "None"
                importance = Importance.None
            },
            Channel().apply {
                id = "min"
                name = "Min"
                importance = Importance.Min
            },
            Channel().apply {
                id = "low"
                name = "Low"
                importance = Importance.Low
            },
            Channel().apply {
                id = "default"
                name = "Default"
                importance = Importance.Default
            },
            Channel().apply {
                id = "high"
                name = "High"
                importance = Importance.High
            }
        )

        assertEquals(5, channels.size)
        assertEquals(Importance.None, channels[0].importance)
        assertEquals(Importance.Min, channels[1].importance)
        assertEquals(Importance.Low, channels[2].importance)
        assertEquals(Importance.Default, channels[3].importance)
        assertEquals(Importance.High, channels[4].importance)
    }

    @Test
    fun testChannel_allVisibilityLevels() {
        val channels = listOf(
            Channel().apply {
                id = "secret"
                name = "Secret"
                visibility = Visibility.Secret
            },
            Channel().apply {
                id = "private"
                name = "Private"
                visibility = Visibility.Private
            },
            Channel().apply {
                id = "public"
                name = "Public"
                visibility = Visibility.Public
            }
        )

        assertEquals(3, channels.size)
        assertEquals(Visibility.Secret, channels[0].visibility)
        assertEquals(Visibility.Private, channels[1].visibility)
        assertEquals(Visibility.Public, channels[2].visibility)
    }

    @Test
    fun testChannel_soundWithExtension() {
        val channel = Channel()
        channel.id = "sound_channel"
        channel.name = "Sound"
        channel.sound = "notification.mp3"

        assertEquals("notification.mp3", channel.sound)
    }

    @Test
    fun testChannel_lightsColorFormats() {
        val channel1 = Channel()
        channel1.id = "hex_color"
        channel1.name = "Hex"
        channel1.lightsColor = "#FF0000"

        val channel2 = Channel()
        channel2.id = "short_hex"
        channel2.name = "Short"
        channel2.lightsColor = "#F00"

        val channel3 = Channel()
        channel3.id = "rgba"
        channel3.name = "RGBA"
        channel3.lightsColor = "#FF0000FF"

        assertEquals("#FF0000", channel1.lightsColor)
        assertEquals("#F00", channel2.lightsColor)
        assertEquals("#FF0000FF", channel3.lightsColor)
    }

    @Test
    fun testChannel_booleanFlags() {
        val channel = Channel()
        channel.id = "flags"
        channel.name = "Flags"
        channel.lights = false
        channel.vibration = false

        assertFalse(channel.lights ?: true)
        assertFalse(channel.vibration ?: true)
    }

    @Test
    fun testDeleteChannelArgs() {
        val args = DeleteChannelArgs()
        args.id = "channel_to_delete"

        assertEquals("channel_to_delete", args.id)
    }

    @Test
    fun testImportance_valueOf() {
        assertEquals(Importance.None, Importance.valueOf("None"))
        assertEquals(Importance.Min, Importance.valueOf("Min"))
        assertEquals(Importance.Low, Importance.valueOf("Low"))
        assertEquals(Importance.Default, Importance.valueOf("Default"))
        assertEquals(Importance.High, Importance.valueOf("High"))
    }

    @Test
    fun testVisibility_valueOf() {
        assertEquals(Visibility.Secret, Visibility.valueOf("Secret"))
        assertEquals(Visibility.Private, Visibility.valueOf("Private"))
        assertEquals(Visibility.Public, Visibility.valueOf("Public"))
    }

    @Test
    fun testChannel_complexConfiguration() {
        val channel = Channel()
        channel.id = "messages"
        channel.name = "Messages"
        channel.description = "Notifications for new messages"
        channel.importance = Importance.High
        channel.visibility = Visibility.Private
        channel.sound = "message_sound"
        channel.vibration = true
        channel.lights = true
        channel.lightsColor = "#0000FF"

        assertEquals("messages", channel.id)
        assertEquals("Messages", channel.name)
        assertEquals("Notifications for new messages", channel.description)
        assertEquals(Importance.High, channel.importance)
        assertEquals(Visibility.Private, channel.visibility)
        assertEquals("message_sound", channel.sound)
        assertTrue(channel.vibration ?: false)
        assertTrue(channel.lights ?: false)
        assertEquals("#0000FF", channel.lightsColor)
    }

    @Test
    fun testChannel_emptyStrings() {
        val channel = Channel()
        channel.id = ""
        channel.name = ""
        channel.description = ""
        channel.sound = ""
        channel.lightsColor = ""

        assertEquals("", channel.id)
        assertEquals("", channel.name)
        assertEquals("", channel.description)
        assertEquals("", channel.sound)
        assertEquals("", channel.lightsColor)
    }
}
