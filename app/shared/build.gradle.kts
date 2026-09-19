// KMP shared app layer: platform-neutral domain models, UI state, and
// frame/pixel abstractions. Compute stays in the Rust `core/` crate.
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
    // Android target: consumed by :app as a plain AAR dependency.
    android {
        namespace = "openllve.shared"
        compileSdk = 36
        minSdk = 26
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
        }
    }

    // iOS targets need Xcode (macOS); enable elsewhere with -Pkmp.ios.enabled=true.
    val enableIosTargets =
        (project.findProperty("kmp.ios.enabled") as? String)?.toBoolean()
            ?: org.gradle.internal.os.OperatingSystem.current().isMacOsX()
    if (enableIosTargets) {
        iosArm64()
        iosSimulatorArm64()
    }
}
