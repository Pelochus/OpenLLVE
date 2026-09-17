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

    composeOptions {
        // Must match the Kotlin compiler version (2.0.21) to avoid the
        // "couldn't find inline method" backend error when inlining Compose
        // functions such as androidx.lifecycle.viewmodel.compose.viewModel.
        kotlinCompilerExtensionVersion = "2.0.21"
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

    // LiteRT (TFLite). Pinned to the classic `org.tensorflow.lite` API (the last
    // non-relocated classic artifact line). 2.17.0 is a relocation POM to
    // `com.google.ai.edge.litert:litert` and does NOT expose the
    // `org.tensorflow.lite` classes this engine is written against. See
    // TODO-app.md §4.1 and docs/DESIGN_SUGGESTIONS.md §5.
    implementation("org.tensorflow:tensorflow-lite:2.14.0")
    implementation("org.tensorflow:tensorflow-lite-gpu:2.14.0")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}
