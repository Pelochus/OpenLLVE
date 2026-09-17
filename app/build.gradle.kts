// AGP 9.x built-in Kotlin: AGP applies the Kotlin Gradle Plugin (KGP)
// internally, so the `org.jetbrains.kotlin.android` plugin is not declared.
// AGP 9.0+ has a runtime dependency on KGP 2.2.10 by default; to compile
// with the latest stable Kotlin (2.4.20), that KGP version is declared on
// this module's buildscript classpath. It must be declared here (not in the
// root build file) so KGP and AGP end up in the same buildscript
// classloader — from the root buildscript, AGP classes are not visible to
// KGP and plugin application fails with a NoClassDefFoundError.
buildscript {
    repositories {
        mavenCentral()
    }
    dependencies {
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.4.20")
    }
}

plugins {
    id("com.android.application")
    // The Compose compiler plugin tracks the KGP version (2.4.20).
    id("org.jetbrains.kotlin.plugin.compose")
}

repositories {
    google()
    mavenCentral()
}

android {
    namespace = "com.example.openllve"
    // compileSdk 36: required by the bumped AndroidX dependencies
    // (e.g. activity-compose 1.13.0 requires compileSdk 36+). targetSdk
    // stays at 34 — compileSdk and targetSdk are independent.
    compileSdk = 36

    defaultConfig {
        applicationId = "com.example.openllve"
        // 26 (Android 8.0): floor for hardware-buffer frame decoding and the
        // NNAPI (NPU) delegate.
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    // AGP 9.x built-in Kotlin: Kotlin source directories must be registered
    // on the `kotlin` source-set (adding them to `java` is no longer
    // supported). The test/androidTest source dirs are empty, so they are
    // not overridden.
    sourceSets {
        getByName("main") {
            kotlin.srcDirs("platforms/android/src/main/java")
            res.srcDirs("platforms/android/src/main/res")
            manifest.srcFile("platforms/android/src/main/AndroidManifest.xml")
            assets.srcDirs("platforms/android/src/main/assets")
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    buildFeatures {
        compose = true
    }

    // Note: from Kotlin 2.0 the Compose compiler is bundled with the Kotlin
    // compiler (via the `org.jetbrains.kotlin.plugin.compose` plugin), so
    // `composeOptions.kotlinCompilerExtensionVersion` is no longer needed.

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packagingOptions {
        resources {
            excludes += "/META-INF/NOTICE"
            excludes += "/META-INF/LICENSE"
        }
    }
}

// With AGP 9.x built-in Kotlin, `kotlin.compilerOptions.jvmTarget` defaults to
// `android.compileOptions.targetCompatibility` (17, set above), so no
// explicit `kotlin { compilerOptions { ... } }` block is needed.

dependencies {
    // Compose BOM 2026.06.01 (Compose UI 1.11.4): the newest BOM that is
    // usable with the newest stable SDK platform (android-36). The newer
    // BOM 2026.08.00 (Compose UI 1.12.0) requires compileSdk 37, which is
    // not yet published in the SDK repository (stable channel tops out at
    // android-36.1).
    val composeBom = platform("androidx.compose:compose-bom:2026.06.01")
    implementation(composeBom)

    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    // Latest stable versions usable with compileSdk 36 (the newest stable SDK
    // platform; android-37 is not yet published). The newer lines
    // (navigation 2.10.x, lifecycle 2.11.x, core 1.19.x, Compose UI 1.12.x)
    // all require compileSdk 37.
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.navigation:navigation-compose:2.9.7")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.10.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
    debugImplementation("androidx.compose.ui:ui-tooling")

    implementation("androidx.core:core-ktx:1.18.0")
    implementation("com.google.android.material:material:1.14.0")
    implementation("androidx.datastore:datastore-preferences:1.2.1")

    // LiteRT 2.2.0 (Google AI Edge runtime) — the modern `CompiledModel` API.
    // The `litert` artifact bundles the native runtime (libLiteRt.so + the GPU
    // accelerator) plus the classic `org.tensorflow.lite` Interpreter classes;
    // the `CompiledModel`/`TensorBuffer`/`Accelerator` API comes from the
    // transitive `litert-api` artifact (which bundles liblitert_jni.so).
    //
    // Exclusions: `litert-api` also pulls the Google Play "ai-delivery" stack
    // (play-services / asset-delivery). We only use the CompiledModel/TensorBuffer
    // path with a model loaded from assets — not the AiPack `ModelProvider`
    // download path — and the only litert-api classes referencing the excluded
    // groups are `ModelProvider`/`ModelSelector`/`AiPackModelProvider`.
    //
    // The `androidx.lifecycle` exclusion that was needed in Change 1 (litert-api
    // pulls lifecycle 2.10.x, which transitively forced Compose UI 1.9.0 /
    // AGP 8.6.0+) is no longer needed: the app now declares lifecycle 2.10.0
    // and Compose BOM 2026.06.01 (UI 1.11.4), so Gradle conflict resolution
    // keeps the app's own versions.
    implementation("com.google.ai.edge.litert:litert:2.2.0") {
        exclude(group = "com.google.android.play")
        exclude(group = "com.google.android.gms")
    }

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.7.0")
}
