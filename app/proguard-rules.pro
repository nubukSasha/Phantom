# Keep UniFFI generated classes
-keep class com.phantom.** { *; }

# Keep Kotlin data classes for serialization
-keepclassmembers class * {
    @kotlinx.serialization.Serializable <fields>;
}

# OkHttp / Conscrypt (if used by Arti)
-dontwarn okhttp3.**
-dontwarn org.conscrypt.**
