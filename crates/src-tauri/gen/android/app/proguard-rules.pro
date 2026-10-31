# Project specific ProGuard rules
-keep class com.gilvave.client.** { *; }
-keep class app.tauri.** { *; }
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
-keepclassmembers class com.gilvave.client.MainActivity {
    public void onWebViewCreate(android.webkit.WebView);
}
-keepattributes *Annotation*,EnclosingMethod,InnerClasses,Signature
