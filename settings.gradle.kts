pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "9.4.0"
        // AGP 9.x enables built-in Kotlin by default: the
        // `org.jetbrains.kotlin.android` plugin is no longer applied (and is
        // incompatible with the AGP 9 new DSL). The Kotlin compiler / Kotlin
        // Gradle Plugin (KGP) version is pinned to 2.4.20 (latest stable)
        // via the buildscript classpath in app/build.gradle.kts.
        // The Compose compiler plugin tracks the KGP version.
        id("org.jetbrains.kotlin.plugin.compose") version "2.4.20"
    }
}

rootProject.name = "OpenLLVE"

// `:app` is the only Gradle module. The Rust core in `core/` is built with
// cargo (see core/README.md), not Gradle.
include(":app")
