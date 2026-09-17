plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

repositories {
    google()
    mavenCentral()
}

android {
    namespace = "com.example.openllve"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.example.openllve"
        // 26 (Android 8.0): floor for hardware-buffer frame decoding and the
        // NNAPI (NPU) delegate. See TODO-app.md for the rationale.
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    sourceSets {
        getByName("main") {
            java.srcDirs("platforms/android/src/main/java")
            res.srcDirs("platforms/android/src/main/res")
            manifest.srcFile("platforms/android/src/main/AndroidManifest.xml")
            assets.srcDirs("platforms/android/src/main/assets")
        }
        getByName("test") {
            java.srcDirs("platforms/android/src/test/java")
        }
        getByName("androidTest") {
            java.srcDirs("platforms/android/src/androidTest/java")
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

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.09.02")
    implementation(composeBom)

    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    implementation("androidx.activity:activity-compose:1.9.2")
    implementation("androidx.navigation:navigation-compose:2.8.5")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    debugImplementation("androidx.compose.ui:ui-tooling")

    implementation("androidx.core:core-ktx:1.12.0")
    implementation("com.google.android.material:material:1.10.0")
    implementation("androidx.datastore:datastore-preferences:1.1.0")

    // LiteRT 2.2.0 (Google AI Edge runtime) — the modern `CompiledModel` API.
    // The `litert` artifact bundles the native runtime (libLiteRt.so + the GPU
    // accelerator) plus the classic `org.tensorflow.lite` Interpreter classes;
    // the `CompiledModel`/`TensorBuffer`/`Accelerator` API comes from the
    // transitive `litert-api` artifact (which bundles liblitert_jni.so).
    // See TODO-app.md §7 (Change 1).
    //
    // Exclusions: `litert-api` also pulls the Google Play "ai-delivery" stack
    // (play-services / asset-delivery) and androidx.lifecycle 2.10.x, which
    // transitively force Compose UI 1.9.0 (requiring AGP 8.6.0+). We only use
    // the CompiledModel/TensorBuffer path with a model loaded from assets — not
    // the AiPack `ModelProvider` download path — and the only litert-api classes
    // referencing the excluded groups are `ModelProvider`/`ModelSelector`/`AiPackModelProvider`.
    // The app keeps its own lifecycle stack (2.8.7 here; bumped in Change 2).
    implementation("com.google.ai.edge.litert:litert:2.2.0") {
        exclude(group = "com.google.android.play")
        exclude(group = "com.google.android.gms")
        exclude(group = "androidx.lifecycle")
    }

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}
