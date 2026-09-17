pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "8.5.2"
        // Kotlin 2.3.0: required by `litert-api:2.2.0` (its classes carry
        // Kotlin 2.3.0 metadata, unreadable by older compilers). This is a
        // prerequisite for the LiteRT migration (TODO-app.md §7 Change 1);
        // the rest of the toolchain bump is Change 2.
        id("org.jetbrains.kotlin.android") version "2.3.0"
        id("org.jetbrains.kotlin.plugin.compose") version "2.3.0"
    }
}

rootProject.name = "OpenLLVE"

// `:app` is the only Gradle module. The Rust core in `core/` is built with
// cargo (see core/README.md), not Gradle.
include(":app")
