import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

val repositoryRoot = rootProject.projectDir.parentFile.parentFile
val generatedJniLibs = layout.buildDirectory.dir("generated/jniLibs")
val requestedTasks = gradle.startParameter.taskNames.joinToString(" ")
val nativeAbis = if (requestedTasks.contains("Emulator")) listOf("x86_64") else listOf("arm64-v8a")
val useWslRenderer = providers.gradleProperty("useWslRenderer").isPresent

// 签名配置从 local.properties（gitignored、机器本地）读取；三项齐备才启用 release
// 签名，否则回退 unsigned（与既有行为一致）。keytool 生成本地 keystore：
// keytool -genkeypair -keystore <path> -alias zeroweb -keyalg RSA -keysize 2048 -validity 10000
val localProperties = Properties().apply {
    val file = rootProject.file("local.properties")
    if (file.exists()) {
        file.inputStream().use { load(it) }
    }
}
val releaseKeystorePath = localProperties.getProperty("zeroweb.keystore.path")
val releaseKeystorePassword = localProperties.getProperty("zeroweb.keystore.password")
val releaseKeystoreAlias = localProperties.getProperty("zeroweb.keystore.alias", "zeroweb")
val releaseSigningReady =
    releaseKeystorePath != null && releaseKeystorePassword != null &&
        rootProject.file(releaseKeystorePath!!).isFile

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

    signingConfigs {
        if (releaseSigningReady) {
            create("release") {
                storeFile = rootProject.file(releaseKeystorePath!!)
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeystoreAlias
                keyPassword = releaseKeystorePassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (releaseSigningReady) {
                signingConfig = signingConfigs.getByName("release")
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
    inputs.file(repositoryRoot.resolve("scripts/android/build-native-wsl.ps1"))
    inputs.file(repositoryRoot.resolve("scripts/android/build-native-wsl.sh"))
    inputs.file(repositoryRoot.resolve("scripts/android/patches/rusty-v8-android-bindgen.patch"))
    inputs.property("useWslRenderer", useWslRenderer)
    outputs.dir(generatedJniLibs)
    doFirst {
        generatedJniLibs.get().asFile.deleteRecursively()
    }
    if (useWslRenderer) {
        require(nativeAbis.size == 1) { "WSL renderer builds require exactly one ABI" }
        // 2026-09-12：Linux/macOS 直调 .sh（env 须含 ZERO_V8_SOURCE 等六项，脚本自校验，
        // Exec 默认继承环境）；Windows 仍走 .ps1 → wsl.exe 转发壳。
        if (System.getProperty("os.name")!!.contains("Windows")) {
            commandLine(
                "powershell",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                repositoryRoot.resolve("scripts/android/build-native-wsl.ps1").absolutePath,
                "-SourceRoot",
                repositoryRoot.absolutePath,
                "-OutputDirectory",
                generatedJniLibs.get().asFile.absolutePath,
                "-Abi",
                nativeAbis.single(),
            )
        } else {
            commandLine(
                "bash",
                repositoryRoot.resolve("scripts/android/build-native-wsl.sh").absolutePath,
                repositoryRoot.absolutePath,
                generatedJniLibs.get().asFile.absolutePath,
                nativeAbis.single(),
            )
        }
    } else {
        environment("V8_FROM_SOURCE", "1")
        commandLine(
            listOf("cargo", "ndk", "-P", "26") +
                nativeAbis.flatMap { listOf("-t", it) } +
                listOf(
                    "-o",
                    generatedJniLibs.get().asFile.absolutePath,
                    "build",
                    "--release",
                    "-p",
                    "zero-android-browser",
                ),
        )
    }
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
