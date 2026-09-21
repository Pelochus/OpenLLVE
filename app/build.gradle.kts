// AGP 9.x applies KGP internally. The KGP version is pinned on this
// module's buildscript classpath (not the root's) so KGP and AGP share the
// same buildscript classloader.
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

    // AGP 9.x: Kotlin sources must be registered on the `kotlin` source set.
    sourceSets {
        getByName("main") {
            kotlin.srcDirs("platforms/android/src/main/java")
            res.srcDirs("platforms/android/src/main/res")
            manifest.srcFile("platforms/android/src/main/AndroidManifest.xml")
            assets.srcDirs("platforms/android/src/main/assets")
        }
    }

    // Release signing is opt-in: scripts/sign-release-apk.sh (the single
    // source of truth) sets the RELEASE_* environment variables consumed
    // here. Without them, release builds fall back to the debug keystore.
    signingConfigs {
        val keystore = providers.environmentVariable("RELEASE_KEYSTORE")
        if (keystore.isPresent) {
            create("release") {
                storeFile = file(keystore.get())
                keyAlias = providers.environmentVariable("RELEASE_KEY_ALIAS").get()
                keyPassword = providers.environmentVariable("RELEASE_KEY_PASSWORD").get()
                storePassword = providers.environmentVariable("RELEASE_STORE_PASSWORD").get()
            }
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // Falls back to the debug keystore unless the RELEASE_* env vars
            // (set by scripts/sign-release-apk.sh) define a release key.
            signingConfig = if (signingConfigs.names.contains("release")) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }

    buildFeatures {
        compose = true
    }

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

dependencies {
    // KMP shared app layer (domain contracts, UI state, frame/pixel abstractions).
    implementation(project(":shared"))

    // Newest BOM usable with compileSdk 36 (2026.08.00 needs compileSdk 37,
    // not yet in the stable SDK repo).
    val composeBom = platform("androidx.compose:compose-bom:2026.06.01")
    implementation(composeBom)

    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    // Newest lines usable with compileSdk 36; the newer ones need compileSdk 37.
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.navigation:navigation-compose:2.9.7")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.10.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
    debugImplementation("androidx.compose.ui:ui-tooling")

    implementation("androidx.core:core-ktx:1.18.0")
    implementation("com.google.android.material:material:1.14.0")
    implementation("androidx.datastore:datastore-preferences:1.2.1")

    // LiteRT 2.2.0 (Google AI Edge) — the modern `CompiledModel` API. The
    // exclusions drop the unused Google Play "ai-delivery" stack (only
    // ModelProvider/ModelSelector reference it; the model loads from assets).
    implementation("com.google.ai.edge.litert:litert:2.2.0") {
        exclude(group = "com.google.android.play")
        exclude(group = "com.google.android.gms")
    }

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.7.0")
}
