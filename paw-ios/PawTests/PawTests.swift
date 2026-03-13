import XCTest
@testable import Paw

final class PawTests: XCTestCase {
    @MainActor
    func testArtifactsDirectoryMatchesGeneratedOutputLocation() {
        XCTAssertEqual(PawCoreManager().artifactsDirectory, "PawCore/Artifacts")
    }
}
