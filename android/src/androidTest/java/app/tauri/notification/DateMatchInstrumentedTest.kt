// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import java.util.*

@RunWith(AndroidJUnit4::class)
class DateMatchInstrumentedTest {

    @Test
    fun testNextTrigger_futureTime() {
        val dateMatch = DateMatch()
        dateMatch.hour = 23
        dateMatch.minute = 59
        dateMatch.second = 0

        val now = Calendar.getInstance()
        now.set(Calendar.HOUR_OF_DAY, 10)
        now.set(Calendar.MINUTE, 0)
        now.set(Calendar.SECOND, 0)
        now.set(Calendar.MILLISECOND, 0)

        val nextTrigger = dateMatch.nextTrigger(now.time)

        assertTrue(nextTrigger > now.timeInMillis)

        val resultCalendar = Calendar.getInstance()
        resultCalendar.timeInMillis = nextTrigger

        assertEquals(23, resultCalendar.get(Calendar.HOUR_OF_DAY))
        assertEquals(59, resultCalendar.get(Calendar.MINUTE))
        assertEquals(0, resultCalendar.get(Calendar.SECOND))
    }

    @Test
    fun testNextTrigger_specificDate() {
        val dateMatch = DateMatch()
        dateMatch.month = Calendar.DECEMBER
        dateMatch.day = 25
        dateMatch.hour = 9
        dateMatch.minute = 0

        val now = Calendar.getInstance()
        now.set(Calendar.MONTH, Calendar.JANUARY)
        now.set(Calendar.DAY_OF_MONTH, 1)
        now.set(Calendar.HOUR_OF_DAY, 12)
        now.set(Calendar.MINUTE, 0)
        now.set(Calendar.SECOND, 0)
        now.set(Calendar.MILLISECOND, 0)

        val nextTrigger = dateMatch.nextTrigger(now.time)

        val resultCalendar = Calendar.getInstance()
        resultCalendar.timeInMillis = nextTrigger

        assertEquals(Calendar.DECEMBER, resultCalendar.get(Calendar.MONTH))
        assertEquals(25, resultCalendar.get(Calendar.DAY_OF_MONTH))
        assertEquals(9, resultCalendar.get(Calendar.HOUR_OF_DAY))
        assertEquals(0, resultCalendar.get(Calendar.MINUTE))
    }

    @Test
    fun testDateMatchSerialization() {
        val original = DateMatch()
        original.year = 2024
        original.month = 11
        original.day = 26
        original.hour = 14
        original.minute = 30
        original.second = 15

        val matchString = original.toMatchString()
        val restored = DateMatch.fromMatchString(matchString)

        assertEquals(original.year, restored.year)
        assertEquals(original.month, restored.month)
        assertEquals(original.day, restored.day)
        assertEquals(original.hour, restored.hour)
        assertEquals(original.minute, restored.minute)
        assertEquals(original.second, restored.second)
    }

    @Test
    fun testDateMatchWithWildcards() {
        val dateMatch = DateMatch()
        dateMatch.hour = 15
        dateMatch.minute = 30
        // Other fields are null (wildcards)

        val matchString = dateMatch.toMatchString()
        assertTrue(matchString.contains("*"))
        assertTrue(matchString.contains("15"))
        assertTrue(matchString.contains("30"))

        val restored = DateMatch.fromMatchString(matchString)
        assertNull(restored.year)
        assertNull(restored.month)
        assertNull(restored.day)
        assertEquals(15, restored.hour)
        assertEquals(30, restored.minute)
    }

    @Test
    fun testGetIntervalTime_allIntervals() {
        val testCases = mapOf(
            NotificationInterval.Second to 1000L,
            NotificationInterval.Minute to 60 * 1000L,
            NotificationInterval.Hour to 60 * 60 * 1000L,
            NotificationInterval.Day to 24 * 60 * 60 * 1000L,
            NotificationInterval.Week to 7 * 24 * 60 * 60 * 1000L,
            NotificationInterval.TwoWeeks to 14 * 24 * 60 * 60 * 1000L
        )

        for ((interval, expectedMillis) in testCases) {
            val result = getIntervalTime(interval, 1)
            assertEquals("Failed for interval: $interval", expectedMillis, result)
        }
    }

    @Test
    fun testGetIntervalTime_multipleCount() {
        assertEquals(3000L, getIntervalTime(NotificationInterval.Second, 3))
        assertEquals(5 * 60 * 1000L, getIntervalTime(NotificationInterval.Minute, 5))
        assertEquals(2 * 60 * 60 * 1000L, getIntervalTime(NotificationInterval.Hour, 2))
        assertEquals(7 * 24 * 60 * 60 * 1000L, getIntervalTime(NotificationInterval.Day, 7))
    }

    @Test
    fun testNotificationScheduleAt_properties() {
        val schedule = NotificationSchedule.At()
        schedule.date = Date()
        schedule.repeating = true
        schedule.allowWhileIdle = true

        assertTrue(schedule.repeating)
        assertTrue(schedule.allowWhileIdle)
        assertFalse(schedule.isRemovable())
        assertTrue(schedule.allowWhileIdle())
    }

    @Test
    fun testNotificationScheduleAt_nonRepeating() {
        val schedule = NotificationSchedule.At()
        schedule.date = Date()
        schedule.repeating = false
        schedule.allowWhileIdle = false

        assertFalse(schedule.repeating)
        assertTrue(schedule.isRemovable())
        assertFalse(schedule.allowWhileIdle())
    }

    @Test
    fun testNotificationScheduleEvery() {
        val schedule = NotificationSchedule.Every()
        schedule.interval = NotificationInterval.Hour
        schedule.count = 2
        schedule.allowWhileIdle = true

        assertEquals(NotificationInterval.Hour, schedule.interval)
        assertEquals(2, schedule.count)
        assertTrue(schedule.allowWhileIdle())
        assertFalse(schedule.isRemovable())
    }

    @Test
    fun testNotificationScheduleInterval() {
        val dateMatch = DateMatch()
        dateMatch.hour = 10
        dateMatch.minute = 30

        val schedule = NotificationSchedule.Interval()
        schedule.interval = dateMatch
        schedule.allowWhileIdle = false

        assertEquals(dateMatch, schedule.interval)
        assertFalse(schedule.allowWhileIdle())
        assertFalse(schedule.isRemovable())
    }

    @Test
    fun testDateMatchEquals() {
        val date1 = DateMatch()
        date1.year = 2024
        date1.month = 11
        date1.day = 26

        val date2 = DateMatch()
        date2.year = 2024
        date2.month = 11
        date2.day = 26

        assertEquals(date1, date2)
        assertEquals(date1.hashCode(), date2.hashCode())
    }

    @Test
    fun testDateMatchNotEquals() {
        val date1 = DateMatch()
        date1.year = 2024
        date1.month = 11

        val date2 = DateMatch()
        date2.year = 2024
        date2.month = 12

        assertNotEquals(date1, date2)
    }

    @Test
    fun testNextTrigger_hourlyRecurrence() {
        val dateMatch = DateMatch()
        dateMatch.minute = 30
        dateMatch.second = 0

        val now = Calendar.getInstance()
        now.set(Calendar.MINUTE, 0)
        now.set(Calendar.SECOND, 0)
        now.set(Calendar.MILLISECOND, 0)

        val nextTrigger = dateMatch.nextTrigger(now.time)

        val resultCalendar = Calendar.getInstance()
        resultCalendar.timeInMillis = nextTrigger

        assertEquals(30, resultCalendar.get(Calendar.MINUTE))
        assertEquals(0, resultCalendar.get(Calendar.SECOND))
    }

    @Test
    fun testNextTrigger_dailyRecurrence() {
        val dateMatch = DateMatch()
        dateMatch.hour = 8
        dateMatch.minute = 0
        dateMatch.second = 0

        val now = Calendar.getInstance()
        now.set(Calendar.HOUR_OF_DAY, 20)
        now.set(Calendar.MINUTE, 0)
        now.set(Calendar.SECOND, 0)
        now.set(Calendar.MILLISECOND, 0)

        val nextTrigger = dateMatch.nextTrigger(now.time)

        assertTrue(nextTrigger > now.timeInMillis)

        val resultCalendar = Calendar.getInstance()
        resultCalendar.timeInMillis = nextTrigger

        assertEquals(8, resultCalendar.get(Calendar.HOUR_OF_DAY))
        assertEquals(0, resultCalendar.get(Calendar.MINUTE))
    }

    @Test
    fun testNotificationInterval_allValues() {
        val intervals = NotificationInterval.values()

        assertEquals(8, intervals.size)
        assertTrue(intervals.contains(NotificationInterval.Year))
        assertTrue(intervals.contains(NotificationInterval.Month))
        assertTrue(intervals.contains(NotificationInterval.TwoWeeks))
        assertTrue(intervals.contains(NotificationInterval.Week))
        assertTrue(intervals.contains(NotificationInterval.Day))
        assertTrue(intervals.contains(NotificationInterval.Hour))
        assertTrue(intervals.contains(NotificationInterval.Minute))
        assertTrue(intervals.contains(NotificationInterval.Second))
    }

    @Test
    fun testScheduleToString() {
        val scheduleAt = NotificationSchedule.At()
        scheduleAt.date = Date(1234567890000L)
        scheduleAt.repeating = true

        val stringAt = scheduleAt.toString()
        assertTrue(stringAt.contains("At"))
        assertTrue(stringAt.contains("repeating=true"))

        val scheduleEvery = NotificationSchedule.Every()
        scheduleEvery.interval = NotificationInterval.Day
        scheduleEvery.count = 3

        val stringEvery = scheduleEvery.toString()
        assertTrue(stringEvery.contains("Every"))
        assertTrue(stringEvery.contains("Day"))
        assertTrue(stringEvery.contains("count=3"))
    }

    @Test
    fun testDateMatchToString() {
        val dateMatch = DateMatch()
        dateMatch.year = 2024
        dateMatch.month = 11
        dateMatch.day = 26
        dateMatch.hour = 14
        dateMatch.minute = 30

        val result = dateMatch.toString()
        assertTrue(result.contains("2024"))
        assertTrue(result.contains("11"))
        assertTrue(result.contains("26"))
        assertTrue(result.contains("14"))
        assertTrue(result.contains("30"))
    }

    @Test
    fun testGetIntervalTime_yearAndMonth() {
        // These are approximations
        val yearMillis = getIntervalTime(NotificationInterval.Year, 1)
        val monthMillis = getIntervalTime(NotificationInterval.Month, 1)

        // Year ≈ 52 weeks
        assertEquals(52 * 7 * 24 * 60 * 60 * 1000L, yearMillis)

        // Month ≈ 30 days
        assertEquals(30 * 24 * 60 * 60 * 1000L, monthMillis)
    }

    @Test
    fun testDateMatchWithWeekday() {
        val dateMatch = DateMatch()
        dateMatch.weekday = Calendar.MONDAY
        dateMatch.hour = 9
        dateMatch.minute = 0

        val matchString = dateMatch.toMatchString()
        val restored = DateMatch.fromMatchString(matchString)

        assertEquals(Calendar.MONDAY, restored.weekday)
        assertEquals(9, restored.hour)
        assertEquals(0, restored.minute)
    }
}
