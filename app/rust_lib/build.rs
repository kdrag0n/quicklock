use std::{env, fs};
use std::path::Path;
use flapigen::{JavaConfig, LanguageConfig};
use rifgen::{Language, TypeCases};

const PACKAGE_NAME: &str = "dev.kdrag0n.quicklock.lib";
const JAVA_LIB_PATH: &str = "../app/src/main/java/dev/kdrag0n/quicklock/lib";

fn main() {
    // Rust side
    let out_dir = env::var("OUT_DIR").unwrap();
    let in_src = "src/java_glue.rs.in";
    let out_src = Path::new(&out_dir).join("java_glue.rs");
    rifgen::Generator::new(TypeCases::CamelCase, Language::Java, "src")
        .generate_interface(in_src);

    let java_dir = Path::new(JAVA_LIB_PATH);
    if java_dir.exists() {
        fs::remove_dir_all(java_dir).unwrap();
    }
    fs::create_dir(java_dir).unwrap();

    // Java side
    let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(
        JavaConfig::new(java_dir.into(), PACKAGE_NAME.into())
            .use_null_annotation_from_package("androidx.annotation".into()),
    )).rustfmt_bindings(true);
    swig_gen.expand("android bindings", &in_src, &out_src);

    println!("cargo:rerun-if-changed=src");
}
