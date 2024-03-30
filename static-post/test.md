## 基本原理
LSPatch会替换掉清单中的appcomponentFactory，并在内部执行其自身的代码。
这是属性是安卓8加入的，所以lspatch的最低版本就是安卓8。
appcomponmentFactory的作用是减少instrumentation的工作量
众所周知，一个安卓程序，第一段被执行的代码就位于instrumentation，instrumentation负责整个程序的内部协作。
instrumentation尤为重量，所以在安卓8，谷歌提供了appcomponentFactory。前者关注程序所有的协作，而后者只关注四大组件的创建。
## 步骤解析
### 第一步就是拷贝LSPatch的源码
https://github.com/LSPosed/LSPatch
### LSPAppComponentFactoryStub
有了上述的介绍，读者们应该清楚LSPatch第一段执行的代码就位于LSPAppComponentFactoryStub，
而他的代码在源码中位于meta-loader中。
大致流程如下
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
            // 获取是哪个架构的

            boolean useManager = false;
            String soPath;

            // 读patch的config
            try (var is = cl.getResourceAsStream(Constants.CONFIG_ASSET_PATH);
                ...
            }

            if (useManager) {
                // 如果使用管理器（本地模式）
                Log.i(TAG, "Bootstrap loader from manager");
                var ipm = IPackageManager.Stub.asInterface(ServiceManager.getService("package"));
                ApplicationInfo manager = ipm.getApplicationInfo(Constants.MANAGER_PACKAGE_NAME, 0, Process.myUid() / 100000);
                try (var zip = new ZipFile(new File(manager.sourceDir));
                     // 从管理器的apk中拿到assets/lspatch/loader.dex
                     var is = zip.getInputStream(zip.getEntry(Constants.LOADER_DEX_ASSET_PATH));
                     var os = new ByteArrayOutputStream()) {
                    transfer(is, os);
                    dex = os.toByteArray();
                }
                soPath = manager.sourceDir + "!/assets/lspatch/so/" + libName + "/liblspatch.so";
            } else {
                Log.i(TAG, "Bootstrap loader from embedment");
                try (var is = cl.getResourceAsStream(Constants.LOADER_DEX_ASSET_PATH);
                     // 从被patch的apk中拿到assets/lspatch/loader.dex
                     var os = new ByteArrayOutputStream()) {
                    transfer(is, os);
                    dex = os.toByteArray();
                }
                soPath = cl.getResource("assets/lspatch/so/" + libName + "/liblspatch.so").getPath().substring(5);
            }
            // 上述流程将会把assets/lspatch/loader.dex读入byte[] dex中
            // 加载lspatch.so
            // 接下来执行patch-loader中的patch_main.cpp
            System.load(soPath);
        } catch (Throwable e) {
            throw new ExceptionInInitializerError(e);
        }
    }
}
```
### patch_main.cpp
上述代码流程很清楚，接下来会加载lspatch.so，而他的源码在patch-loader
大致流程如下
```java

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
        // 在core模块中，目的是防止修改内存时死锁
        InitSymbolCache(nullptr);
        // 初始化lsplant的api
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

        // 拿到刚才的byte[] dex，并转为lsp的内部dex格式（加载到匿名内存段规避检测）
        ScopedLocalRef<jbyteArray> array = JNI_GetStaticObjectField(env, stub, dex_field);
        auto dex = PreloadedDex {env->GetByteArrayElements(array.get(), nullptr), static_cast<size_t>(JNI_GetArrayLength(env, array))};

        // 执行lsplant的hook
        InitArtHooker(env, initInfo);
        // 通过InMemoryDexClassLoader加载dex
        LoadDex(env, std::move(dex));
        // 进行签名校验和openat的bypass
        InitHooks(env);

        GetArt(true);

        SetupEntryClass(env);
        // 接下来执行org.lsposed.lspatch.loader.LSPApplication的void onLoad
        FindAndCall(env, "onLoad", "()V");
    }
} // namespace lspd

```
逻辑很简单，就两步：
1. 把appcomponefactory中的byte[] dex放到匿名内存，并通过inmemeory加载。
2. 调用org.lsposed.lspatch.loader.LSPApplication的void onLoad
### LSPApplication
其源码位于patch-loader中
大致流程如下
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
我觉得这段代码并没有很晦涩难懂，所以我不做过多细节介绍。
这里Onload主要做了6步：
1. 首先判断是否是isolated_process，如果是的话什么都不执行
2. 创建一个新的LoadedApk，并创建一个新的context
3. 调用context.getClassLoader().loadClass(config.appComponentFactory);加载原本的appComponentFactory
4. 随后使用新的context，禁用Profile，并模拟Startup操作
5. 加载modules
6. 将上面创建的fake-loadedApk中的classloader设置为real-loadedApk

至此LSPatch的面纱已经彻底揭开
## 总结
LSPatch使用了匿名内存加载自己的loader文件，这样做很安全
LSPatch对open系的函数进行了hook，使得应用不能读到patch后的信息
LSPatch创建了fake-LoadedApk，并使用他创建了AppContext，随后调用了原始的appcomponmentFactory，导致软件无法通过查看classloader来检测注入

## 如何检测
最简单的就是instrumentation。除此自外还可以尝试SVC直接syscall调用open。
甚至可以通过判断profile是否正常加载来检测
