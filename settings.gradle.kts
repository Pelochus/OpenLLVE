pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "8.5.2"
        id("org.jetbrains.kotlin.android") version "2.0.21"
        id("org.jetbrains.kotlin.plugin.compose") version "2.0.21"
    }
}

rootProject.name = "OpenLLVE"

// `:app` is the only Gradle module. The Rust core in `core/` is built with
// cargo (see core/README.md), not Gradle.
include(":app")
