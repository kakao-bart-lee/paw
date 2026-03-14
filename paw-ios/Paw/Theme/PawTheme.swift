import SwiftUI

enum PawTheme {
    static let background = Color(red: 0.017, green: 0.019, blue: 0.017)
    static let backgroundGlow = Color(red: 0.088, green: 0.064, blue: 0.042)
    static let surface1 = Color(red: 0.024, green: 0.025, blue: 0.024)
    static let surface2 = Color(red: 0.041, green: 0.042, blue: 0.040)
    static let surface3 = Color(red: 0.061, green: 0.062, blue: 0.058)
    static let outline = Color.white.opacity(0.06)

    static let strongText = Color(red: 0.83, green: 0.81, blue: 0.78)
    static let mutedText = Color(red: 0.42, green: 0.40, blue: 0.38)
    static let subtleText = Color(red: 0.56, green: 0.53, blue: 0.50)

    static let teal = Color(red: 0.45, green: 0.83, blue: 0.79)
    static let tealSoft = Color(red: 0.10, green: 0.20, blue: 0.19)
    static let amber = Color(red: 0.76, green: 0.56, blue: 0.31)
    static let amberSoft = Color(red: 0.17, green: 0.12, blue: 0.08)
    static let lavender = Color(red: 0.60, green: 0.57, blue: 0.90)
    static let coral = Color(red: 0.84, green: 0.63, blue: 0.61)

    static let success = teal
    static let warning = amber
    static let danger = Color(red: 0.80, green: 0.38, blue: 0.36)

    static let cardShadow = Color.black.opacity(0.32)
}
