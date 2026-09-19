// KMP shared app layer (P1.3): platform-neutral domain models, UI state
// contracts, and pixel/frame abstractions shared by the Android and (future)
// iOS platforms. Performance-sensitive compute stays in the Rust `core/`
// crate; this module holds app-side contracts only.
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    // AGP 9: the KMP-compatible Android library plugin (replaces
    // `com.android.library`, which is incompatible with KMP on AGP 9.0+).
    id("com.android.kotlin.multiplatform.library")
}

repositories {
    mavenCentral()
}

kotlin {
    // Android target: consumed by :app (AGP) as a plain AAR dependency.
    // On AGP 9 the Android target config lives in `kotlin { android { ... } }`.
    android {
        namespace = "openllve.shared"
        compileSdk = 36
        minSdk = 26
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }

    // iOS targets: only buildable on macOS (Xcode is required). They are
    // declared by default on macOS hosts; on other hosts enable them with
    // -Pkmp.ios.enabled=true if the Kotlin/Native toolchain is available.
    val enableIosTargets =
        (project.findProperty("kmp.ios.enabled") as? String)?.toBoolean()
            ?: org.gradle.internal.os.OperatingSystem.current().isMacOsX()
    if (enableIosTargets) {
        iosArm64()
        iosSimulatorArm64()
    }
}
