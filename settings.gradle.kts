pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "9.4.0"
        // AGP 9 KMP-compatible Android library plugin for the :shared module.
        id("com.android.kotlin.multiplatform.library") version "9.4.0"
        // Compose compiler plugin tracks the KGP version (pinned via the
        // buildscript classpath in app/build.gradle.kts).
        id("org.jetbrains.kotlin.plugin.compose") version "2.4.20"
        id("org.jetbrains.kotlin.multiplatform") version "2.4.20"
    }
}

rootProject.name = "OpenLLVE"

// The Rust core in `core/` is built with cargo, not Gradle.
include(":app")
include(":shared")
project(":shared").projectDir = file("app/shared")
