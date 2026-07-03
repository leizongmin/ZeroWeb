plugins {
    id("com.android.application")
}

android {
    namespace = "com.zeroweb.ui"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.zeroweb.ui"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    // 不需要额外依赖——纯 NativeActivity 模式
}
