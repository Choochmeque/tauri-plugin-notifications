import XCTest
import StoreKit
import StoreKitTest
@testable import tauri_plugin_iap

// MARK: - Test Helpers

/// Parses a JSON string into a dictionary.
private func parseJSON(_ jsonString: String) -> JsonObject? {
    guard let data = jsonString.data(using: .utf8),
          let json = try? JSONSerialization.jsonObject(with: data) as? JsonObject else {
        return nil
    }
    return json
}

final class PluginTests: XCTestCase {
    var plugin: IapPlugin!

    override func setUp() {
        super.setUp()
        plugin = initPlugin()
    }

    override func tearDown() {
        plugin = nil
        super.tearDown()
    }

    // MARK: - PurchaseStateValue Tests

    func testPurchaseStateValueRawValues() {
        XCTAssertEqual(PurchaseStateValue.purchased.rawValue, 0)
        XCTAssertEqual(PurchaseStateValue.canceled.rawValue, 1)
        XCTAssertEqual(PurchaseStateValue.pending.rawValue, 2)
    }

    func testPurchaseStateValueFromRawValue() {
        XCTAssertEqual(PurchaseStateValue(rawValue: 0), .purchased)
        XCTAssertEqual(PurchaseStateValue(rawValue: 1), .canceled)
        XCTAssertEqual(PurchaseStateValue(rawValue: 2), .pending)
        XCTAssertNil(PurchaseStateValue(rawValue: 99))
    }

    // MARK: - Initialize Tests

    func testInitialize() throws {
        let jsonString = try plugin.initialize()
        let json = try XCTUnwrap(parseJSON(jsonString))
        XCTAssertEqual(json["success"] as? Bool, true)
    }

    func testInitializeIsIdempotent() throws {
        for _ in 0..<5 {
            XCTAssertNoThrow(try plugin.initialize())
        }
    }

    func testInitializeReturnsValidJSON() throws {
        let jsonString = try plugin.initialize()
        let data = try XCTUnwrap(jsonString.data(using: .utf8))
        XCTAssertNoThrow(try JSONSerialization.jsonObject(with: data))
    }

    // MARK: - AcknowledgePurchase Tests

    func testAcknowledgePurchase() async throws {
        let jsonString = try await plugin.acknowledgePurchase(purchaseToken: RustString("test_token"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        XCTAssertEqual(json["success"] as? Bool, true)
    }

    func testAcknowledgePurchaseWithEmptyToken() async throws {
        _ = try await plugin.acknowledgePurchase(purchaseToken: RustString(""))
    }

    func testAcknowledgePurchaseWithLongToken() async throws {
        let longToken = String(repeating: "token_", count: 100)
        _ = try await plugin.acknowledgePurchase(purchaseToken: RustString(longToken))
    }

    func testAcknowledgePurchaseReturnsValidJSON() async throws {
        let jsonString = try await plugin.acknowledgePurchase(purchaseToken: RustString("any_token"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        XCTAssertTrue(json.keys.contains("success"))
    }
}

// MARK: - StoreKit Integration Tests

@available(macOS 12.0, *)
final class StoreKitTests: XCTestCase {
    var session: SKTestSession!
    var plugin: IapPlugin!

    override func setUp() async throws {
        try await super.setUp()

        plugin = initPlugin()

        let url = try XCTUnwrap(
            Bundle.module.url(forResource: "TestProducts", withExtension: "storekit")
        )

        session = try SKTestSession(contentsOf: url)
        session.resetToDefaultState()
        session.disableDialogs = true
        session.clearTransactions()
    }

    override func tearDown() async throws {
        session.clearTransactions()
        session = nil
        plugin = nil
        try await super.tearDown()
    }

    // MARK: - getProducts Tests

    func testGetProductsReturnsProducts() async throws {
        // TODO: fix it somehow
        throw XCTSkip("Skipping due to StoreKit daemon unavailability")

        let productIds = RustVec<RustString>()
        productIds.push(value: RustString("com.test.removeads"))
        productIds.push(value: RustString("com.test.premium"))

        let jsonString = try await plugin.getProducts(productIds: productIds, productType: RustString("inapp"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        let products = try XCTUnwrap(json["products"] as? [JsonObject])

        XCTAssertEqual(products.count, 2)

        if let firstProduct = products.first {
            XCTAssertNotNil(firstProduct["productId"])
            XCTAssertNotNil(firstProduct["title"])
            XCTAssertNotNil(firstProduct["description"])
            XCTAssertNotNil(firstProduct["formattedPrice"])
        }
    }

    func testGetProductsWithSubscription() async throws {
        // TODO: fix it somehow
        throw XCTSkip("Skipping due to StoreKit daemon unavailability")

        let productIds = RustVec<RustString>()
        productIds.push(value: RustString("com.test.premium.monthly"))

        let jsonString = try await plugin.getProducts(productIds: productIds, productType: RustString("subs"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        let products = try XCTUnwrap(json["products"] as? [JsonObject])

        XCTAssertEqual(products.count, 1)

        if let subscription = products.first {
            XCTAssertEqual(subscription["productId"] as? String, "com.test.premium.monthly")
            XCTAssertNotNil(subscription["subscriptionOfferDetails"])
        }
    }

    func testGetProductsWithNonExistentProduct() async throws {
        let productIds = RustVec<RustString>()
        productIds.push(value: RustString("com.test.nonexistent"))

        let jsonString = try await plugin.getProducts(productIds: productIds, productType: RustString("inapp"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        let products = try XCTUnwrap(json["products"] as? [JsonObject])

        XCTAssertEqual(products.count, 0)
    }

    func testGetProductsWithEmptyArray() async throws {
        let productIds = RustVec<RustString>()

        let jsonString = try await plugin.getProducts(productIds: productIds, productType: RustString("inapp"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        let products = try XCTUnwrap(json["products"] as? [JsonObject])

        XCTAssertEqual(products.count, 0)
    }

    func testGetProductsWithConsumable() async throws {
        // TODO: fix it somehow
        throw XCTSkip("Skipping due to StoreKit daemon unavailability")

        let productIds = RustVec<RustString>()
        productIds.push(value: RustString("com.test.coins100"))

        let jsonString = try await plugin.getProducts(productIds: productIds, productType: RustString("inapp"))
        let json = try XCTUnwrap(parseJSON(jsonString))
        let products = try XCTUnwrap(json["products"] as? [JsonObject])

        XCTAssertEqual(products.count, 1)

        if let product = products.first {
            XCTAssertEqual(product["productId"] as? String, "com.test.coins100")
            XCTAssertEqual(product["title"] as? String, "100 Coins")
        }
    }
}
