// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import org.junit.Assert.*
import org.junit.Test
import java.util.*

class DateMatchTest {

    @Test
    fun testDateMatch_equals_identical() {
        val date1 = DateMatch()
        date1.year = 2024
        date1.month = 5
        date1.day = 15

        val date2 = DateMatch()
        date2.year = 2024
        date2.month = 5
        date2.day = 15

        assertEquals(date1, date2)
    }

    @Test
    fun testDateMatch_equals_different() {
        val date1 = DateMatch()
        date1.year = 2024
        date1.month = 5

        val date2 = DateMatch()
        date2.year = 2024
        date2.month = 6

        assertNotEquals(date1, date2)
    }

    @Test
    fun testDateMatch_equals_sameInstance() {
        val date = DateMatch()
        date.year = 2024

        assertEquals(date, date)
    }

    @Test
    fun testDateMatch_equals_null() {
        val date = DateMatch()
        date.year = 2024

        assertNotEquals(date, null)
    }

    @Test
    fun testDateMatch_hashCode_identical() {
        val date1 = DateMatch()
        date1.year = 2024
        date1.month = 5
        date1.hour = 10

        val date2 = DateMatch()
        date2.year = 2024
        date2.month = 5
        date2.hour = 10

        assertEquals(date1.hashCode(), date2.hashCode())
    }

    @Test
    fun testDateMatch_hashCode_different() {
        val date1 = DateMatch()
        date1.year = 2024

        val date2 = DateMatch()
        date2.year = 2025

        assertNotEquals(date1.hashCode(), date2.hashCode())
    }

    @Test
    fun testDateMatch_toString() {
        val date = DateMatch()
        date.year = 2024
        date.month = 11
        date.day = 26
        date.hour = 14
        date.minute = 30
        date.second = 0

        val result = date.toString()
        assertTrue(result.contains("year=2024"))
        assertTrue(result.contains("month=11"))
        assertTrue(result.contains("day=26"))
        assertTrue(result.contains("hour=14"))
        assertTrue(result.contains("minute=30"))
        assertTrue(result.contains("second=0"))
    }

    @Test
    fun testDateMatch_toMatchString_allFields() {
        val date = DateMatch()
        date.year = 2024
        date.month = 5
        date.day = 15
        date.weekday = 2
        date.hour = 10
        date.minute = 30
        date.second = 0
        date.unit = Calendar.HOUR_OF_DAY

        val result = date.toMatchString()
        assertEquals("2024 5 15 2 10 30 0 11", result)
    }

    @Test
    fun testDateMatch_toMatchString_partialFields() {
        val date = DateMatch()
        date.hour = 10
        date.minute = 30

        val result = date.toMatchString()
        assertTrue(result.contains("*"))
        assertTrue(result.contains("10"))
        assertTrue(result.contains("30"))
    }

    @Test
    fun testDateMatch_fromMatchString_allFields() {
        val matchString = "2024 5 15 2 10 30 0 11"
        val date = DateMatch.fromMatchString(matchString)

        assertEquals(2024, date.year)
        assertEquals(5, date.month)
        assertEquals(15, date.day)
        assertEquals(2, date.weekday)
        assertEquals(10, date.hour)
        assertEquals(30, date.minute)
        assertEquals(0, date.second)
        assertEquals(11, date.unit)
    }

    @Test
    fun testDateMatch_fromMatchString_withWildcards() {
        val matchString = "* * 15 * 10 30 * -1"
        val date = DateMatch.fromMatchString(matchString)

        assertNull(date.year)
        assertNull(date.month)
        assertEquals(15, date.day)
        assertNull(date.weekday)
        assertEquals(10, date.hour)
        assertEquals(30, date.minute)
        assertNull(date.second)
        assertEquals(-1, date.unit)
    }

    @Test
    fun testDateMatch_fromMatchString_sevenFields() {
        val matchString = "2024 5 15 2 10 30 11"
        val date = DateMatch.fromMatchString(matchString)

        assertEquals(2024, date.year)
        assertEquals(5, date.month)
        assertEquals(15, date.day)
        assertEquals(2, date.weekday)
        assertEquals(10, date.hour)
        assertEquals(30, date.minute)
        assertEquals(11, date.unit)
        assertNull(date.second)
    }

    @Test
    fun testDateMatch_nextTrigger_futureDate() {
        val date = DateMatch()
        date.hour = 23
        date.minute = 59

        val calendar = Calendar.getInstance()
        calendar.set(Calendar.HOUR_OF_DAY, 10)
        calendar.set(Calendar.MINUTE, 0)
        calendar.set(Calendar.SECOND, 0)
        calendar.set(Calendar.MILLISECOND, 0)

        val baseDate = calendar.time
        val nextTrigger = date.nextTrigger(baseDate)

        assertTrue(nextTrigger > baseDate.time)
    }

    @Test
    fun testDateMatch_nextTrigger_specificHourAndMinute() {
        val date = DateMatch()
        date.hour = 15
        date.minute = 30

        val calendar = Calendar.getInstance()
        calendar.set(Calendar.HOUR_OF_DAY, 10)
        calendar.set(Calendar.MINUTE, 0)
        calendar.set(Calendar.SECOND, 0)
        calendar.set(Calendar.MILLISECOND, 0)

        val baseDate = calendar.time
        val nextTrigger = date.nextTrigger(baseDate)

        val resultCalendar = Calendar.getInstance()
        resultCalendar.timeInMillis = nextTrigger

        assertEquals(15, resultCalendar.get(Calendar.HOUR_OF_DAY))
        assertEquals(30, resultCalendar.get(Calendar.MINUTE))
    }

    @Test
    fun testGetIntervalTime_year() {
        val result = getIntervalTime(NotificationInterval.Year, 1)
        assertEquals(52 * 7 * 24 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_month() {
        val result = getIntervalTime(NotificationInterval.Month, 1)
        assertEquals(30 * 24 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_twoWeeks() {
        val result = getIntervalTime(NotificationInterval.TwoWeeks, 1)
        assertEquals(14 * 24 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_week() {
        val result = getIntervalTime(NotificationInterval.Week, 1)
        assertEquals(7 * 24 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_day() {
        val result = getIntervalTime(NotificationInterval.Day, 1)
        assertEquals(24 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_hour() {
        val result = getIntervalTime(NotificationInterval.Hour, 1)
        assertEquals(60 * 60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_minute() {
        val result = getIntervalTime(NotificationInterval.Minute, 1)
        assertEquals(60 * 1000L, result)
    }

    @Test
    fun testGetIntervalTime_second() {
        val result = getIntervalTime(NotificationInterval.Second, 1)
        assertEquals(1000L, result)
    }

    @Test
    fun testGetIntervalTime_multipleCount() {
        val result = getIntervalTime(NotificationInterval.Hour, 3)
        assertEquals(3 * 60 * 60 * 1000L, result)
    }

    @Test
    fun testDateMatch_roundTripSerialization() {
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
}
