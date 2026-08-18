plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

val repositoryRoot = rootProject.projectDir.parentFile.parentFile
val generatedJniLibs = layout.buildDirectory.dir("generated/jniLibs")

android {
    namespace = "com.leizm.zeroweb"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.leizm.zeroweb"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    flavorDimensions += "abi"
    productFlavors {
        create("arm64") {
            dimension = "abi"
            ndk {
                abiFilters += "arm64-v8a"
            }
        }
        create("emulator") {
            dimension = "abi"
            ndk {
                abiFilters += "x86_64"
            }
        }
    }

    buildFeatures {
        aidl = true
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main").jniLibs.directories.add(generatedJniLibs.get().asFile.absolutePath)
    }
}

androidComponents {
    beforeVariants(selector().withBuildType("release")) { variantBuilder ->
        if (variantBuilder.productFlavors.any { it.second == "emulator" }) {
            variantBuilder.enable = false
        }
    }
}

val buildRustNative by tasks.registering(Exec::class) {
    group = "build"
    description = "Build ZeroWeb Android native libraries for release and emulator validation ABIs."
    workingDir = repositoryRoot
    inputs.dir(repositoryRoot.resolve("apps/android-browser/rust"))
    inputs.file(repositoryRoot.resolve("Cargo.toml"))
    outputs.dir(generatedJniLibs)
    doFirst {
        generatedJniLibs.get().asFile.deleteRecursively()
    }
    commandLine(
        "cargo",
        "ndk",
        "-P",
        "26",
        "-t",
        "arm64-v8a",
        "-t",
        "x86_64",
        "-o",
        generatedJniLibs.get().asFile.absolutePath,
        "build",
        "--release",
        "-p",
        "zero-android-browser",
    )
}

tasks.configureEach {
    if (name.startsWith("merge") && (name.endsWith("NativeLibs") || name.endsWith("JniLibFolders"))) {
        dependsOn(buildRustNative)
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    debugImplementation(libs.androidx.compose.ui.tooling)
}
