fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=payload/cliply.exe.gz");
    println!("cargo:rerun-if-changed=payload/cliply.ico");
    println!("cargo:rerun-if-env-changed=CLIPLY_INSTALLER_TEST_MANIFEST");

    #[cfg(windows)]
    {
        let execution_level = if std::env::var_os("CLIPLY_INSTALLER_TEST_MANIFEST").is_some() {
            "asInvoker"
        } else {
            "requireAdministrator"
        };
        let manifest = format!(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="{execution_level}" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#
        );
        let windows = tauri_build::WindowsAttributes::new().app_manifest(&manifest);
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        tauri_build::try_build(attrs).expect("failed to run tauri build script");
    }

    #[cfg(not(windows))]
    tauri_build::build();
}
