buildscript {
    repositories {
        google()
        mavenCentral()
    }
    dependencies {
        classpath("com.android.tools.build:gradle:9.3.1")
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:2.2.10")
    }
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

// Align Android library subprojects (such as tauri-android) with an installed SDK platform
// if their default compileSdk is not present locally.
subprojects {
    afterEvaluate {
        extensions.findByType(com.android.build.gradle.BaseExtension::class.java)?.let { androidExt ->
            val sdkDir = androidExt.sdkDirectory
            val desired = androidExt.compileSdkVersion
            if (desired != null && !java.io.File(sdkDir, "platforms/$desired").exists()) {
                val installed = listOf(37, 36, 35).firstOrNull { api ->
                    java.io.File(sdkDir, "platforms/android-$api").exists() ||
                        java.io.File(sdkDir, "platforms/android-$api.0").exists()
                }
                if (installed != null) {
                    androidExt.compileSdkVersion(installed)
                }
            }
        }
    }
}

tasks.register("clean").configure {
    delete("build")
}
