package app.tauri.notification

import org.junit.Assert.*
import org.junit.Test
import java.util.*

class NotificationScheduleTest {

    @Test
    fun testScheduleAt_isRemovable_nonRepeating() {
        val schedule = NotificationSchedule.At()
        schedule.date = Date()
        schedule.repeating = false

        assertTrue(schedule.isRemovable())
    }

    @Test
    fun testScheduleAt_isRemovable_repeating() {
        val schedule = NotificationSchedule.At()
        schedule.date = Date()
        schedule.repeating = true

        assertFalse(schedule.isRemovable())
    }

    @Test
    fun testScheduleInterval_isRemovable() {
        val schedule = NotificationSchedule.Interval()
        schedule.interval = DateMatch()

        assertFalse(schedule.isRemovable())
    }

    @Test
    fun testScheduleEvery_isRemovable() {
        val schedule = NotificationSchedule.Every()
        schedule.interval = NotificationInterval.Hour
        schedule.count = 1

        assertFalse(schedule.isRemovable())
    }

    @Test
    fun testScheduleAt_allowWhileIdle_true() {
        val schedule = NotificationSchedule.At()
        schedule.allowWhileIdle = true

        assertTrue(schedule.allowWhileIdle())
    }

    @Test
    fun testScheduleAt_allowWhileIdle_false() {
        val schedule = NotificationSchedule.At()
        schedule.allowWhileIdle = false

        assertFalse(schedule.allowWhileIdle())
    }

    @Test
    fun testScheduleInterval_allowWhileIdle() {
        val schedule = NotificationSchedule.Interval()
        schedule.allowWhileIdle = true
        schedule.interval = DateMatch()

        assertTrue(schedule.allowWhileIdle())
    }

    @Test
    fun testScheduleEvery_allowWhileIdle() {
        val schedule = NotificationSchedule.Every()
        schedule.allowWhileIdle = true
        schedule.interval = NotificationInterval.Day
        schedule.count = 1

        assertTrue(schedule.allowWhileIdle())
    }

    @Test
    fun testScheduleAt_toString() {
        val schedule = NotificationSchedule.At()
        val date = Date(1234567890000L)
        schedule.date = date
        schedule.repeating = true
        schedule.allowWhileIdle = true

        val result = schedule.toString()
        assertTrue(result.contains("At"))
        assertTrue(result.contains("repeating=true"))
        assertTrue(result.contains("allowWhileIdle=true"))
    }

    @Test
    fun testScheduleInterval_toString() {
        val schedule = NotificationSchedule.Interval()
        val dateMatch = DateMatch()
        dateMatch.hour = 10
        schedule.interval = dateMatch
        schedule.allowWhileIdle = false

        val result = schedule.toString()
        assertTrue(result.contains("Interval"))
        assertTrue(result.contains("allowWhileIdle=false"))
    }

    @Test
    fun testScheduleEvery_toString() {
        val schedule = NotificationSchedule.Every()
        schedule.interval = NotificationInterval.Week
        schedule.count = 2
        schedule.allowWhileIdle = true

        val result = schedule.toString()
        assertTrue(result.contains("Every"))
        assertTrue(result.contains("interval=Week"))
        assertTrue(result.contains("count=2"))
        assertTrue(result.contains("allowWhileIdle=true"))
    }
}
