import Foundation
@testable import tauri_plugin_iap

// MARK: - Mock Storage

private var mockStringStorage: [UnsafeMutableRawPointer: String] = [:]
private var nextMockPtr: UInt = 1

private func allocateMockPtr() -> UnsafeMutableRawPointer {
    let ptr = UnsafeMutableRawPointer(bitPattern: nextMockPtr)!
    nextMockPtr += 1
    return ptr
}

// MARK: - RustString Mock FFI

@_cdecl("__swift_bridge__$RustString$new")
func mock_RustString_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockStringStorage[ptr] = ""
    return ptr
}

@_cdecl("__swift_bridge__$RustString$new_with_str")
func mock_RustString_new_with_str(_ str: RustStr) -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    let swiftString = str.toString()
    mockStringStorage[ptr] = swiftString
    return ptr
}

@_cdecl("__swift_bridge__$RustString$_free")
func mock_RustString_free(_ ptr: UnsafeMutableRawPointer) {
    mockStringStorage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$RustString$len")
func mock_RustString_len(_ ptr: UnsafeMutableRawPointer) -> UInt {
    guard let str = mockStringStorage[ptr] else { return 0 }
    return UInt(str.utf8.count)
}

@_cdecl("__swift_bridge__$RustString$as_str")
func mock_RustString_as_str(_ ptr: UnsafeMutableRawPointer) -> RustStr {
    guard let str = mockStringStorage[ptr] else {
        return RustStr(start: nil, len: 0)
    }
    let utf8 = Array(str.utf8)
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: utf8.count)
    buffer.initialize(from: utf8, count: utf8.count)
    return RustStr(start: buffer, len: UInt(utf8.count))
}

@_cdecl("__swift_bridge__$RustString$trim")
func mock_RustString_trim(_ ptr: UnsafeMutableRawPointer) -> RustStr {
    guard let str = mockStringStorage[ptr] else {
        return RustStr(start: nil, len: 0)
    }
    let trimmed = str.trimmingCharacters(in: .whitespacesAndNewlines)
    let utf8 = Array(trimmed.utf8)
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: utf8.count)
    buffer.initialize(from: utf8, count: utf8.count)
    return RustStr(start: buffer, len: UInt(utf8.count))
}

@_cdecl("__swift_bridge__$RustStr$partial_eq")
func mock_RustStr_partial_eq(_ lhs: RustStr, _ rhs: RustStr) -> Bool {
    return lhs.toString() == rhs.toString()
}

// MARK: - RustVec<RustString> Mock FFI

private var mockVecStringStorage: [UnsafeMutableRawPointer: [UnsafeMutableRawPointer]] = [:]

@_cdecl("__swift_bridge__$Vec_RustString$new")
func mock_Vec_RustString_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecStringStorage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_RustString$drop")
func mock_Vec_RustString_drop(_ ptr: UnsafeMutableRawPointer) {
    mockVecStringStorage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_RustString$push")
func mock_Vec_RustString_push(_ vecPtr: UnsafeMutableRawPointer, _ itemPtr: UnsafeMutableRawPointer) {
    mockVecStringStorage[vecPtr]?.append(itemPtr)
}

@_cdecl("__swift_bridge__$Vec_RustString$pop")
func mock_Vec_RustString_pop(_ vecPtr: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    return mockVecStringStorage[vecPtr]?.popLast()
}

@_cdecl("__swift_bridge__$Vec_RustString$get")
func mock_Vec_RustString_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> UnsafeMutableRawPointer? {
    guard let vec = mockVecStringStorage[vecPtr], index < vec.count else { return nil }
    return vec[Int(index)]
}

@_cdecl("__swift_bridge__$Vec_RustString$get_mut")
func mock_Vec_RustString_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> UnsafeMutableRawPointer? {
    return mock_Vec_RustString_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_RustString$len")
func mock_Vec_RustString_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecStringStorage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_RustString$as_ptr")
func mock_Vec_RustString_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafeRawPointer? {
    guard let vec = mockVecStringStorage[vecPtr], !vec.isEmpty else { return nil }
    return UnsafeRawPointer(vec.withUnsafeBufferPointer { $0.baseAddress })
}

// MARK: - Null pointer helper

@_cdecl("__swift_bridge__null_pointer")
func mock_null_pointer() -> UnsafeMutableRawPointer? {
    return nil
}

// MARK: - Boxed FnOnce helpers

@_cdecl("__swift_bridge__$call_boxed_fn_once_no_args_no_return")
func mock_call_boxed_fn_once(_ ptr: UnsafeMutableRawPointer) {
    // No-op for tests
}

@_cdecl("__swift_bridge__$free_boxed_fn_once_no_args_no_return")
func mock_free_boxed_fn_once(_ ptr: UnsafeMutableRawPointer) {
    // No-op for tests
}

// MARK: - Generic Vec Storage for primitive types

private var mockVecU8Storage: [UnsafeMutableRawPointer: [UInt8]] = [:]
private var mockVecU16Storage: [UnsafeMutableRawPointer: [UInt16]] = [:]
private var mockVecU32Storage: [UnsafeMutableRawPointer: [UInt32]] = [:]
private var mockVecU64Storage: [UnsafeMutableRawPointer: [UInt64]] = [:]
private var mockVecUsizeStorage: [UnsafeMutableRawPointer: [UInt]] = [:]
private var mockVecI8Storage: [UnsafeMutableRawPointer: [Int8]] = [:]
private var mockVecI16Storage: [UnsafeMutableRawPointer: [Int16]] = [:]
private var mockVecI32Storage: [UnsafeMutableRawPointer: [Int32]] = [:]
private var mockVecI64Storage: [UnsafeMutableRawPointer: [Int64]] = [:]
private var mockVecIsizeStorage: [UnsafeMutableRawPointer: [Int]] = [:]
private var mockVecBoolStorage: [UnsafeMutableRawPointer: [Bool]] = [:]
private var mockVecF32Storage: [UnsafeMutableRawPointer: [Float]] = [:]
private var mockVecF64Storage: [UnsafeMutableRawPointer: [Double]] = [:]

// MARK: - Vec<u8>

@_cdecl("__swift_bridge__$Vec_u8$new")
func mock_Vec_u8_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecU8Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_u8$_free")
func mock_Vec_u8_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecU8Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_u8$push")
func mock_Vec_u8_push(_ vecPtr: UnsafeMutableRawPointer, _ val: UInt8) {
    mockVecU8Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_u8$pop")
func mock_Vec_u8_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionU8 {
    if let val = mockVecU8Storage[vecPtr]?.popLast() {
        return __private__OptionU8(val: val, is_some: true)
    }
    return __private__OptionU8(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u8$get")
func mock_Vec_u8_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU8 {
    if let vec = mockVecU8Storage[vecPtr], index < vec.count {
        return __private__OptionU8(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionU8(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u8$get_mut")
func mock_Vec_u8_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU8 {
    return mock_Vec_u8_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_u8$len")
func mock_Vec_u8_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecU8Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_u8$as_ptr")
func mock_Vec_u8_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UInt8>? {
    return mockVecU8Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<u16>

@_cdecl("__swift_bridge__$Vec_u16$new")
func mock_Vec_u16_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecU16Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_u16$_free")
func mock_Vec_u16_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecU16Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_u16$push")
func mock_Vec_u16_push(_ vecPtr: UnsafeMutableRawPointer, _ val: UInt16) {
    mockVecU16Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_u16$pop")
func mock_Vec_u16_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionU16 {
    if let val = mockVecU16Storage[vecPtr]?.popLast() {
        return __private__OptionU16(val: val, is_some: true)
    }
    return __private__OptionU16(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u16$get")
func mock_Vec_u16_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU16 {
    if let vec = mockVecU16Storage[vecPtr], index < vec.count {
        return __private__OptionU16(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionU16(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u16$get_mut")
func mock_Vec_u16_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU16 {
    return mock_Vec_u16_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_u16$len")
func mock_Vec_u16_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecU16Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_u16$as_ptr")
func mock_Vec_u16_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UInt16>? {
    return mockVecU16Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<u32>

@_cdecl("__swift_bridge__$Vec_u32$new")
func mock_Vec_u32_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecU32Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_u32$_free")
func mock_Vec_u32_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecU32Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_u32$push")
func mock_Vec_u32_push(_ vecPtr: UnsafeMutableRawPointer, _ val: UInt32) {
    mockVecU32Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_u32$pop")
func mock_Vec_u32_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionU32 {
    if let val = mockVecU32Storage[vecPtr]?.popLast() {
        return __private__OptionU32(val: val, is_some: true)
    }
    return __private__OptionU32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u32$get")
func mock_Vec_u32_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU32 {
    if let vec = mockVecU32Storage[vecPtr], index < vec.count {
        return __private__OptionU32(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionU32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u32$get_mut")
func mock_Vec_u32_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU32 {
    return mock_Vec_u32_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_u32$len")
func mock_Vec_u32_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecU32Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_u32$as_ptr")
func mock_Vec_u32_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UInt32>? {
    return mockVecU32Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<u64>

@_cdecl("__swift_bridge__$Vec_u64$new")
func mock_Vec_u64_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecU64Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_u64$_free")
func mock_Vec_u64_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecU64Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_u64$push")
func mock_Vec_u64_push(_ vecPtr: UnsafeMutableRawPointer, _ val: UInt64) {
    mockVecU64Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_u64$pop")
func mock_Vec_u64_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionU64 {
    if let val = mockVecU64Storage[vecPtr]?.popLast() {
        return __private__OptionU64(val: val, is_some: true)
    }
    return __private__OptionU64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u64$get")
func mock_Vec_u64_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU64 {
    if let vec = mockVecU64Storage[vecPtr], index < vec.count {
        return __private__OptionU64(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionU64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_u64$get_mut")
func mock_Vec_u64_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionU64 {
    return mock_Vec_u64_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_u64$len")
func mock_Vec_u64_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecU64Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_u64$as_ptr")
func mock_Vec_u64_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UInt64>? {
    return mockVecU64Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<usize>

@_cdecl("__swift_bridge__$Vec_usize$new")
func mock_Vec_usize_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecUsizeStorage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_usize$_free")
func mock_Vec_usize_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecUsizeStorage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_usize$push")
func mock_Vec_usize_push(_ vecPtr: UnsafeMutableRawPointer, _ val: UInt) {
    mockVecUsizeStorage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_usize$pop")
func mock_Vec_usize_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionUsize {
    if let val = mockVecUsizeStorage[vecPtr]?.popLast() {
        return __private__OptionUsize(val: val, is_some: true)
    }
    return __private__OptionUsize(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_usize$get")
func mock_Vec_usize_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionUsize {
    if let vec = mockVecUsizeStorage[vecPtr], index < vec.count {
        return __private__OptionUsize(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionUsize(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_usize$get_mut")
func mock_Vec_usize_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionUsize {
    return mock_Vec_usize_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_usize$len")
func mock_Vec_usize_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecUsizeStorage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_usize$as_ptr")
func mock_Vec_usize_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<UInt>? {
    return mockVecUsizeStorage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<i8>

@_cdecl("__swift_bridge__$Vec_i8$new")
func mock_Vec_i8_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecI8Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_i8$_free")
func mock_Vec_i8_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecI8Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_i8$push")
func mock_Vec_i8_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Int8) {
    mockVecI8Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_i8$pop")
func mock_Vec_i8_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionI8 {
    if let val = mockVecI8Storage[vecPtr]?.popLast() {
        return __private__OptionI8(val: val, is_some: true)
    }
    return __private__OptionI8(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i8$get")
func mock_Vec_i8_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI8 {
    if let vec = mockVecI8Storage[vecPtr], index < vec.count {
        return __private__OptionI8(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionI8(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i8$get_mut")
func mock_Vec_i8_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI8 {
    return mock_Vec_i8_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_i8$len")
func mock_Vec_i8_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecI8Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_i8$as_ptr")
func mock_Vec_i8_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Int8>? {
    return mockVecI8Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<i16>

@_cdecl("__swift_bridge__$Vec_i16$new")
func mock_Vec_i16_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecI16Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_i16$_free")
func mock_Vec_i16_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecI16Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_i16$push")
func mock_Vec_i16_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Int16) {
    mockVecI16Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_i16$pop")
func mock_Vec_i16_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionI16 {
    if let val = mockVecI16Storage[vecPtr]?.popLast() {
        return __private__OptionI16(val: val, is_some: true)
    }
    return __private__OptionI16(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i16$get")
func mock_Vec_i16_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI16 {
    if let vec = mockVecI16Storage[vecPtr], index < vec.count {
        return __private__OptionI16(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionI16(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i16$get_mut")
func mock_Vec_i16_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI16 {
    return mock_Vec_i16_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_i16$len")
func mock_Vec_i16_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecI16Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_i16$as_ptr")
func mock_Vec_i16_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Int16>? {
    return mockVecI16Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<i32>

@_cdecl("__swift_bridge__$Vec_i32$new")
func mock_Vec_i32_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecI32Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_i32$_free")
func mock_Vec_i32_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecI32Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_i32$push")
func mock_Vec_i32_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Int32) {
    mockVecI32Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_i32$pop")
func mock_Vec_i32_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionI32 {
    if let val = mockVecI32Storage[vecPtr]?.popLast() {
        return __private__OptionI32(val: val, is_some: true)
    }
    return __private__OptionI32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i32$get")
func mock_Vec_i32_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI32 {
    if let vec = mockVecI32Storage[vecPtr], index < vec.count {
        return __private__OptionI32(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionI32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i32$get_mut")
func mock_Vec_i32_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI32 {
    return mock_Vec_i32_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_i32$len")
func mock_Vec_i32_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecI32Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_i32$as_ptr")
func mock_Vec_i32_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Int32>? {
    return mockVecI32Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<i64>

@_cdecl("__swift_bridge__$Vec_i64$new")
func mock_Vec_i64_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecI64Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_i64$_free")
func mock_Vec_i64_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecI64Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_i64$push")
func mock_Vec_i64_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Int64) {
    mockVecI64Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_i64$pop")
func mock_Vec_i64_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionI64 {
    if let val = mockVecI64Storage[vecPtr]?.popLast() {
        return __private__OptionI64(val: val, is_some: true)
    }
    return __private__OptionI64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i64$get")
func mock_Vec_i64_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI64 {
    if let vec = mockVecI64Storage[vecPtr], index < vec.count {
        return __private__OptionI64(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionI64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_i64$get_mut")
func mock_Vec_i64_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionI64 {
    return mock_Vec_i64_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_i64$len")
func mock_Vec_i64_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecI64Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_i64$as_ptr")
func mock_Vec_i64_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Int64>? {
    return mockVecI64Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<isize>

@_cdecl("__swift_bridge__$Vec_isize$new")
func mock_Vec_isize_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecIsizeStorage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_isize$_free")
func mock_Vec_isize_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecIsizeStorage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_isize$push")
func mock_Vec_isize_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Int) {
    mockVecIsizeStorage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_isize$pop")
func mock_Vec_isize_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionIsize {
    if let val = mockVecIsizeStorage[vecPtr]?.popLast() {
        return __private__OptionIsize(val: val, is_some: true)
    }
    return __private__OptionIsize(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_isize$get")
func mock_Vec_isize_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionIsize {
    if let vec = mockVecIsizeStorage[vecPtr], index < vec.count {
        return __private__OptionIsize(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionIsize(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_isize$get_mut")
func mock_Vec_isize_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionIsize {
    return mock_Vec_isize_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_isize$len")
func mock_Vec_isize_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecIsizeStorage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_isize$as_ptr")
func mock_Vec_isize_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Int>? {
    return mockVecIsizeStorage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<bool>

@_cdecl("__swift_bridge__$Vec_bool$new")
func mock_Vec_bool_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecBoolStorage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_bool$_free")
func mock_Vec_bool_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecBoolStorage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_bool$push")
func mock_Vec_bool_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Bool) {
    mockVecBoolStorage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_bool$pop")
func mock_Vec_bool_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionBool {
    if let val = mockVecBoolStorage[vecPtr]?.popLast() {
        return __private__OptionBool(val: val, is_some: true)
    }
    return __private__OptionBool(val: false, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_bool$get")
func mock_Vec_bool_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionBool {
    if let vec = mockVecBoolStorage[vecPtr], index < vec.count {
        return __private__OptionBool(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionBool(val: false, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_bool$get_mut")
func mock_Vec_bool_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionBool {
    return mock_Vec_bool_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_bool$len")
func mock_Vec_bool_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecBoolStorage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_bool$as_ptr")
func mock_Vec_bool_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Bool>? {
    return mockVecBoolStorage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<f32>

@_cdecl("__swift_bridge__$Vec_f32$new")
func mock_Vec_f32_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecF32Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_f32$_free")
func mock_Vec_f32_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecF32Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_f32$push")
func mock_Vec_f32_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Float) {
    mockVecF32Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_f32$pop")
func mock_Vec_f32_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionF32 {
    if let val = mockVecF32Storage[vecPtr]?.popLast() {
        return __private__OptionF32(val: val, is_some: true)
    }
    return __private__OptionF32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_f32$get")
func mock_Vec_f32_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionF32 {
    if let vec = mockVecF32Storage[vecPtr], index < vec.count {
        return __private__OptionF32(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionF32(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_f32$get_mut")
func mock_Vec_f32_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionF32 {
    return mock_Vec_f32_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_f32$len")
func mock_Vec_f32_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecF32Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_f32$as_ptr")
func mock_Vec_f32_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Float>? {
    return mockVecF32Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Vec<f64>

@_cdecl("__swift_bridge__$Vec_f64$new")
func mock_Vec_f64_new() -> UnsafeMutableRawPointer {
    let ptr = allocateMockPtr()
    mockVecF64Storage[ptr] = []
    return ptr
}

@_cdecl("__swift_bridge__$Vec_f64$_free")
func mock_Vec_f64_free(_ ptr: UnsafeMutableRawPointer) {
    mockVecF64Storage.removeValue(forKey: ptr)
}

@_cdecl("__swift_bridge__$Vec_f64$push")
func mock_Vec_f64_push(_ vecPtr: UnsafeMutableRawPointer, _ val: Double) {
    mockVecF64Storage[vecPtr]?.append(val)
}

@_cdecl("__swift_bridge__$Vec_f64$pop")
func mock_Vec_f64_pop(_ vecPtr: UnsafeMutableRawPointer) -> __private__OptionF64 {
    if let val = mockVecF64Storage[vecPtr]?.popLast() {
        return __private__OptionF64(val: val, is_some: true)
    }
    return __private__OptionF64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_f64$get")
func mock_Vec_f64_get(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionF64 {
    if let vec = mockVecF64Storage[vecPtr], index < vec.count {
        return __private__OptionF64(val: vec[Int(index)], is_some: true)
    }
    return __private__OptionF64(val: 0, is_some: false)
}

@_cdecl("__swift_bridge__$Vec_f64$get_mut")
func mock_Vec_f64_get_mut(_ vecPtr: UnsafeMutableRawPointer, _ index: UInt) -> __private__OptionF64 {
    return mock_Vec_f64_get(vecPtr, index)
}

@_cdecl("__swift_bridge__$Vec_f64$len")
func mock_Vec_f64_len(_ vecPtr: UnsafeMutableRawPointer) -> UInt {
    return UInt(mockVecF64Storage[vecPtr]?.count ?? 0)
}

@_cdecl("__swift_bridge__$Vec_f64$as_ptr")
func mock_Vec_f64_as_ptr(_ vecPtr: UnsafeMutableRawPointer) -> UnsafePointer<Double>? {
    return mockVecF64Storage[vecPtr]?.withUnsafeBufferPointer { $0.baseAddress }
}

// MARK: - Trigger Mock (Rust FFI callback)

@_cdecl("__swift_bridge__$trigger")
func mock_trigger(_ event: UnsafeMutableRawPointer, _ payload: UnsafeMutableRawPointer) {
    // No-op stub for tests
}
