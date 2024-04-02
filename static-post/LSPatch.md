# How LSPatch works

What is LSPatch?
> LSPatch: A non-root Xposed framework extending from LSPosed.

While the LSPatch is amazing, there's no magic behind the technology. In this post, we will analyze it's source code, understand how it works and find ways to anagist it.


## Before we start

Most developers think that the first line of code executed by an Android app is located in Application.

That's not entirely true. Manifest.xml has a label named "Instrumentation" which is the real class that developer can customize. But even though it's the real entrypoint, no one uses it because of it's heavy responsibility. 

For the above reason, Google publishd  __AppComponentFactory__, a class used only to create _The Five Components_, since Android O (8.0). It's cooresponding label in AndroidManifest is __appcomponentFactory__.

> The Five Components: Activity; Service; Broadcast; ContentProvider; Application;

## Let's go!

First of all, we should clone it's source code, and make a simple demo.

> AndroidManifest.xml
```xml
<?xml version='1.0' encoding='utf-8'?>
<manifest
    xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.mycompany.application2">
    <application>
    </application>
</manifest>
```
That's all, enough.

Then use LSPatch to process it. And finally, unpack it.
_ps. Do not worry about wich mode to use, there's no difference._

> processed AndroidManifest.xml
```xml
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.mycompany.application2"
    platformBuildVersionCode="30"
    platformBuildVersionName="11"
    android:versionCode="1"
    android:versionName="1.0"
    android:compileSdkVersion="30"
    android:compileSdkVersionCodename="11">
    <uses-sdk
        android:minSdkVersion="28"
        android:targetSdkVersion="23" />
    <application
        android:debuggable="true"
        android:appComponentFactory="org.lsposed.lspatch.metaloader.LSPAppComponentFactoryStub">
        <meta-data
            android:name="lspatch"
            android:value="..." />
            <!-- I omit this value -->
    </application>
    <uses-permission android:name="android.permission.QUERY_ALL_PACKAGES" />
</manifest>
```

From the processed apk, you will find that LSPatch releaces your origin class of appcomponentFactory to the LSPAppComponentFactoryStub. 

Obviously, this is where the magic comes from.

---

### LSPAppComponentFactoryStub

This class is located in module <mark>meta-loader</mark>.
```java
public class LSPAppComponentFactoryStub extends AppComponentFactory {

    private static final String TAG = "LSPatch-MetaLoader";
    private static final Map<String, String> archToLib = new HashMap<String, String>(4);

    public static byte[] dex;

    static {
        try {
            archToLib.put("arm", "armeabi-v7a");
            archToLib.put("arm64", "arm64-v8a");
            archToLib.put("x86", "x86");
            archToLib.put("x86_64", "x86_64");

            ...
            String libName = archToLib.get(arch);
            // fetch device architecture

            boolean useManager = false;
            String soPath;

            // read config of patch
            try (var is = cl.getResourceAsStream(Constants.CONFIG_ASSET_PATH);
                ...
            }

            if (useManager) {
                // manager mode
                Log.i(TAG, "Bootstrap loader from manager");
                var ipm = IPackageManager.Stub.asInterface(ServiceManager.getService("package"));
                ApplicationInfo manager = ipm.getApplicationInfo(Constants.MANAGER_PACKAGE_NAME, 0, Process.myUid() / 100000);
                try (var zip = new ZipFile(new File(manager.sourceDir));
                     // copy assets/lspatch/loader.dex from manager
                     var is = zip.getInputStream(zip.getEntry(Constants.LOADER_DEX_ASSET_PATH));
                     var os = new ByteArrayOutputStream()) {
                    transfer(is, os);
                    dex = os.toByteArray();
                }
                soPath = manager.sourceDir + "!/assets/lspatch/so/" + libName + "/liblspatch.so";
            } else {
                // local mode
                Log.i(TAG, "Bootstrap loader from embedment");
                try (var is = cl.getResourceAsStream(Constants.LOADER_DEX_ASSET_PATH);
                     // copy assets/lspatch/loader.dex from self
                     var os = new ByteArrayOutputStream()) {
                    transfer(is, os);
                    dex = os.toByteArray();
                }
                soPath = cl.getResource("assets/lspatch/so/" + libName + "/liblspatch.so").getPath().substring(5);
            }
            // After above processing, assets/lspatch/loader.dex is readed into byte[] dex
            // Then, load lspatch.so
            System.load(soPath);
        } catch (Throwable e) {
            throw new ExceptionInInitializerError(e);
        }
    }
}
```
Summing-ip:
The first job LSPAppComponentFactoryStub does is read dex, according to patch's config.
The second job is to call System.load, and load lspatch.so.

---

### lspatch.so

The source code of lspatch.so is patch_main.cpp, located in module <mark>patch-loader</mark>.

```cpp
#include <jni.h>

#include "config_impl.h"
#include "patch_loader.h"

JNIEXPORT jint JNI_OnLoad(JavaVM* vm, void* reserved) {
    JNIEnv* env;
    if (vm->GetEnv(reinterpret_cast<void**>(&env), JNI_VERSION_1_6) != JNI_OK) {
        return JNI_ERR;
    }
    lspd::PatchLoader::Init();
    lspd::ConfigImpl::Init();
    lspd::PatchLoader::GetInstance()->Load(env);
    return JNI_VERSION_1_6;
}
```
Looks like there are 2 important classes - PatchLoader and ConfigImpl.

ConfigImpl is very simple, so we analyze it first.

#### ConfigImpl

```cpp
#pragma once

#include <string>
#include "config_bridge.h"

namespace lspd {

    class ConfigImpl : public ConfigBridge {
    public:
        inline static void Init() {
            instance_ = std::make_unique<ConfigImpl>();
        }

        virtual obfuscation_map_t& obfuscation_map() override {
            return obfuscation_map_;
        }

        virtual void obfuscation_map(obfuscation_map_t m) override {
            obfuscation_map_ = std::move(m);
        }

    private:
        inline static std::map<std::string, std::string> obfuscation_map_ = {
                {"de.robv.android.xposed.", "de.robv.android.xposed."},
                { "android.app.AndroidApp", "android.app.AndroidApp"},
                { "android.content.res.XRes", "android.content.res.XRes"},
                { "android.content.res.XModule", "android.content.res.XModule"},
                { "org.lsposed.lspd.core.", "org.lsposed.lspd.core."},
                { "org.lsposed.lspd.nativebridge.", "org.lsposed.lspd.nativebridge."},
                { "org.lsposed.lspd.service.", "org.lsposed.lspd.service."},
        };
    };
}
```

The location of config_bridge.h is in LSPosed's module named <mark>core</mark>.

```cpp
#pragma once

#include <map>

namespace lspd {
    using obfuscation_map_t = std::map<std::string, std::string>;

    class ConfigBridge {
    public:
        inline static ConfigBridge *GetInstance() {
            return instance_.get();
        }

        inline static std::unique_ptr<ConfigBridge> ReleaseInstance() {
            return std::move(instance_);
        }

        virtual obfuscation_map_t &obfuscation_map() = 0;

        virtual void obfuscation_map(obfuscation_map_t) = 0;

        virtual ~ConfigBridge() = default;

    protected:
        static std::unique_ptr<ConfigBridge> instance_;
    };
}
```

This class uses the singleton pattern, and maintians a hash map internally.

Very simple.

#### PatchLoader

PatchLoader contains .h and .cpp.

> PatchLoader.h

```cpp

#include "context.h"

namespace lspd {
    inline lsplant::InitInfo handler;
    class PatchLoader : public Context {
    public:
        inline static void Init() {
            instance_ = std::make_unique<PatchLoader>();
        }
        inline static PatchLoader* GetInstance() {
            return static_cast<PatchLoader*>(instance_.get());
        }
        void Load(JNIEnv* env);
    protected:
        void InitArtHooker(JNIEnv* env, const lsplant::InitInfo& initInfo) override;
        void InitHooks(JNIEnv* env) override;
        void LoadDex(JNIEnv* env, PreloadedDex&& dex) override;
        void SetupEntryClass(JNIEnv* env) override;
    };
} // namespace lspd
```

Also, the location of context.h is in LSPosed's module named <mark>core</mark>.

We don't need to know it's details.

> PatchLoader.cpp

```cpp
namespace lspd {

    void PatchLoader::LoadDex(JNIEnv* env, Context::PreloadedDex&& dex) {
        ...
        auto in_memory_classloader = JNI_FindClass(env, "dalvik/system/InMemoryDexClassLoader");
        auto mid_init = JNI_GetMethodID(env, in_memory_classloader, "<init>",
                                        "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V");
        auto byte_buffer_class = JNI_FindClass(env, "java/nio/ByteBuffer");
        auto dex_buffer = env->NewDirectByteBuffer(dex.data(), dex.size());
        if (auto my_cl = JNI_NewObject(env, in_memory_classloader, mid_init, dex_buffer, stub_classloader)) {
            inject_class_loader_ = JNI_NewGlobalRef(env, my_cl);
        }
        env->DeleteLocalRef(dex_buffer);
    }

    void PatchLoader::SetupEntryClass(JNIEnv* env) {
        if (auto entry_class = FindClassFromLoader(env, GetCurrentClassLoader(),
                                                   "org.lsposed.lspatch.loader.LSPApplication")) {
            entry_class_ = JNI_NewGlobalRef(env, entry_class);
        }
    }

    void PatchLoader::Load(JNIEnv* env) {
        // Located in module core. To avoid dead-lock when modify memory.
        InitSymbolCache(nullptr);
        // Config of LSPlant.
        lsplant::InitInfo initInfo {
                .inline_hooker = [](auto t, auto r) {
                    void* bk = nullptr;
                    return HookFunction(t, r, &bk) == RS_SUCCESS ? bk : nullptr;
                },
                .inline_unhooker = [](auto t) {
                    return UnhookFunction(t) == RT_SUCCESS;
                },
                .art_symbol_resolver = [](auto symbol) {
                    return GetArt()->getSymbAddress<void*>(symbol);
                },
                .art_symbol_prefix_resolver = [](auto symbol) {
                    return GetArt()->getSymbPrefixFirstAddress(symbol);
                },
        };

        auto stub = JNI_FindClass(env, "org/lsposed/lspatch/metaloader/LSPAppComponentFactoryStub");
        auto dex_field = JNI_GetStaticFieldID(env, stub, "dex", "[B");
        // get byte[] dex in LSPAppComponentFactoryStub, and convert it to PreloadedDex
        // There is one important thing, in the constructor of PreloadedDex, byte[] dex will be moved to anonymous memory alloced by mmap.
        ScopedLocalRef<jbyteArray> array = JNI_GetStaticObjectField(env, stub, dex_field);
        auto dex = PreloadedDex {env->GetByteArrayElements(array.get(), nullptr), static_cast<size_t>(JNI_GetArrayLength(env, array))};

        // Initialize LSPlant's hook
        InitArtHooker(env, initInfo);
        // load dex by InMemoryDexClassLoader
        LoadDex(env, std::move(dex));
        // do hook
        InitHooks(env);

        GetArt(true);

        SetupEntryClass(env);
        // call "void onLoad" in org.lsposed.lspatch.loader.LSPApplication
        FindAndCall(env, "onLoad", "()V");
    }
} // namespace lspd

```

Note here that LSPatch uses a means to avoid detection.
LSPatch's dex will be loaded into anonymous memory.

> PreloadedDex

```cpp
    Context::PreloadedDex::PreloadedDex(int fd, std::size_t size) {
        LOGD("Context::PreloadedDex::PreloadedDex: fd={}, size={}", fd, size);
        auto *addr = mmap(nullptr, size, PROT_READ, MAP_SHARED, fd, 0);

        if (addr != MAP_FAILED) {
            addr_ = addr;
            size_ = size;
        } else {
            PLOGE("Read dex");
        }
    }

    Context::PreloadedDex::~PreloadedDex() {
        if (*this) munmap(addr_, size_);
    } 
```

---

### LSPApplication

The source code locates in module <mark>patch-loader</mark>.

In fact, although it's name is Application, it is not _Application_.

```java
public class LSPApplication {
    private static ActivityThread activityThread;
    private static LoadedApk stubLoadedApk;
    private static LoadedApk appLoadedApk;

    private static PatchConfig config;

    public static void onLoad() throws RemoteException, IOException {
        if (isIsolated()) {
            return;
        }
        activityThread = ActivityThread.currentActivityThread();
        var context = createLoadedApkWithContext();
        ILSPApplicationService service;
        if (config.useManager) {
            service = new RemoteApplicationService(context);
        } else {
            service = new LocalApplicationService(context);
        }

        disableProfile(context);
        Startup.initXposed(false, ActivityThread.currentProcessName(), context.getApplicationInfo().dataDir, service);
        Startup.bootstrapXposed();
        LSPLoader.initModules(appLoadedApk);

        switchAllClassLoader();
        SigBypass.doSigBypass(context, config.sigBypassLevel);
    }

    private static Context createLoadedApkWithContext() {
        try {
            var mBoundApplication = XposedHelpers.getObjectField(activityThread, "mBoundApplication");

            stubLoadedApk = (LoadedApk) XposedHelpers.getObjectField(mBoundApplication, "info");
            var appInfo = (ApplicationInfo) XposedHelpers.getObjectField(mBoundApplication, "appInfo");
            var compatInfo = (CompatibilityInfo) XposedHelpers.getObjectField(mBoundApplication, "compatInfo");
            var baseClassLoader = stubLoadedApk.getClassLoader();

            try (var is = baseClassLoader.getResourceAsStream(CONFIG_ASSET_PATH)) {
                BufferedReader streamReader = new BufferedReader(new InputStreamReader(is, StandardCharsets.UTF_8));
                config = new Gson().fromJson(streamReader, PatchConfig.class);
            } catch (IOException e) {
                Log.e(TAG, "Failed to load config file");
                return null;
            }
            Log.i(TAG, "Use manager: " + config.useManager);
            Log.i(TAG, "Signature bypass level: " + config.sigBypassLevel);

            Path originPath = Paths.get(appInfo.dataDir, "cache/lspatch/origin/");
            Path cacheApkPath;
            try (ZipFile sourceFile = new ZipFile(appInfo.sourceDir)) {
                cacheApkPath = originPath.resolve(sourceFile.getEntry(ORIGINAL_APK_ASSET_PATH).getCrc() + ".apk");
            }

            appInfo.sourceDir = cacheApkPath.toString();
            appInfo.publicSourceDir = cacheApkPath.toString();
            appInfo.appComponentFactory = config.appComponentFactory;

            if (!Files.exists(cacheApkPath)) {
                Log.i(TAG, "Extract original apk");
                FileUtils.deleteFolderIfExists(originPath);
                Files.createDirectories(originPath);
                try (InputStream is = baseClassLoader.getResourceAsStream(ORIGINAL_APK_ASSET_PATH)) {
                    Files.copy(is, cacheApkPath);
                }
            }
            cacheApkPath.toFile().setWritable(false);

            var mPackages = (Map<?, ?>) XposedHelpers.getObjectField(activityThread, "mPackages");
            mPackages.remove(appInfo.packageName);
            appLoadedApk = activityThread.getPackageInfoNoCheck(appInfo, compatInfo);
            XposedHelpers.setObjectField(mBoundApplication, "info", appLoadedApk);

            var activityClientRecordClass = XposedHelpers.findClass("android.app.ActivityThread$ActivityClientRecord", ActivityThread.class.getClassLoader());
            var fixActivityClientRecord = (BiConsumer<Object, Object>) (k, v) -> {
                if (activityClientRecordClass.isInstance(v)) {
                    var pkgInfo = XposedHelpers.getObjectField(v, "packageInfo");
                    if (pkgInfo == stubLoadedApk) {
                        Log.d(TAG, "fix loadedapk from ActivityClientRecord");
                        XposedHelpers.setObjectField(v, "packageInfo", appLoadedApk);
                    }
                }
            };
            var mActivities = (Map<?, ?>) XposedHelpers.getObjectField(activityThread, "mActivities");
            mActivities.forEach(fixActivityClientRecord);
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    var mLaunchingActivities = (Map<?, ?>) XposedHelpers.getObjectField(activityThread, "mLaunchingActivities");
                    mLaunchingActivities.forEach(fixActivityClientRecord);
                }
            } catch (Throwable ignored) {
            }
            Log.i(TAG, "hooked app initialized: " + appLoadedApk);

            var context = (Context) XposedHelpers.callStaticMethod(Class.forName("android.app.ContextImpl"), "createAppContext", activityThread, stubLoadedApk);
            if (config.appComponentFactory != null) {
                try {
                    context.getClassLoader().loadClass(config.appComponentFactory);
                } catch (ClassNotFoundException e) { // This will happen on some strange shells like 360
                    Log.w(TAG, "Original AppComponentFactory not found: " + config.appComponentFactory);
                    appInfo.appComponentFactory = null;
                }
            }
            return context;
        } catch (Throwable e) {
            Log.e(TAG, "createLoadedApk", e);
            return null;
        }
    }
    private static void switchAllClassLoader() {
        var fields = LoadedApk.class.getDeclaredFields();
        for (Field field : fields) {
            if (field.getType() == ClassLoader.class) {
                var obj = XposedHelpers.getObjectField(appLoadedApk, field.getName());
                XposedHelpers.setObjectField(stubLoadedApk, field.getName(), obj);
            }
        }
    }
}

```

I don't think this part is hard to understand, it only does six thing.

1. check for _isolated_process_
2. make a instance of _LoadedApk_, and use it to create a _Context_
3. call _context.getClassLoader().loadClass(config.appComponentFactory);_ and  set it to LoadedApk
4. disable _Profile_ and call _Startup_ using fake-context.
5. loaded Xposed's modules
6. replace all _classloader_ in fake-loadedApk to classloader in real-loadedApk

---

## Final summing-up

All of the above processes run in static block of LSPAppComponentFactoryStub.

Processed app cann't know what happened.

The magic of LSPatch is fully revealed.

## Ways to anagist it

The most simple method is to add _instrumentation_.
In addition, using _SVC_ can avoid LSPatch' hook.
Fair enough, you can _check memory_ in /proc/self/maps.
