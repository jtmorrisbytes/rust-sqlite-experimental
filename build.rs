use std::{fs::File, io::{BufRead, BufReader, Write}, path::PathBuf, process::Command};



use std::collections::HashSet; // <-- THE FIX: Import a HashSet to track uniqueness

fn generate_opcodes() {
    println!("cargo:rerun-if-changed=c_src/vdbe.c");
    
    let vdbe_path = "c_src/vdbe.c";
    let output_path = "c_src/opcodes.h";
    
    let file = File::open(vdbe_path).expect("Failed to open vdbe.c for opcode generation");
    let reader = BufReader::new(file);
    let mut out_file = File::create(output_path).expect("Failed to create opcodes.h");
    
    // Set up our uniqueness filter tracking layout
    let mut seen_opcodes = HashSet::new();
    
    writeln!(out_file, "#ifndef OPCODES_H").unwrap();
    writeln!(out_file, "#define OPCODES_H\n").unwrap();
    
    let mut opcode_count = 1;
    
    for line in reader.lines() {
        if let Ok(text) = line {
            // Find execution case indicators in the state machine
            if text.contains("case OP_") && text.contains(':') {
                if let Some(start) = text.find("OP_") {
                    if let Some(end) = text[start..].find(':') {
                        let opcode_name = text[start..start + end].trim();
                        
                        // SANITIZATION FIX: Only write the macro if it hasn't been parsed yet!
                        if !seen_opcodes.contains(opcode_name) {
                            writeln!(out_file, "#define {:<25} {}", opcode_name, opcode_count).unwrap();
                            seen_opcodes.insert(opcode_name.to_string());
                            opcode_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    writeln!(out_file, "\n#endif /* OPCODES_H */").unwrap();
}




fn main() {
    // // 1. Tell Cargo to rerun this script ONLY if the C files change
    // println!("cargo:rerun-if-changed=c_src");
    // println!("cargo:rerun-if-changed=c_src");

    // // 1. Get Cargo's isolated build output directory
    // let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // let lemon_exe_name = if cfg!(windows) { "lemon.exe" } else { "lemon" };
    // let lemon_exe_path = out_dir.join(lemon_exe_name);

    // // 2. Use CC to compile lemon.c automatically
    // let compiler = cc::Build::new();
    // let compiler_cmd = compiler.get_compiler();
    
    // let mut cc_cmd = Command::new(compiler_cmd.path());
    
    // // Add compiler args to output the executable to OUT_DIR
    // if compiler_cmd.is_like_msvc() {
    //     cc_cmd.args(&["c_src/lemon.c", &format!("/Fe:{}", lemon_exe_path.display())]);
    // } else {
    //     cc_cmd.args(&["c_src/lemon.c", "-o", &lemon_exe_path.to_string_lossy()]);
    // }

    // // Set any environment variables CC needs (like target flags)
    // for (key, val) in compiler_cmd.env() {
    //     cc_cmd.env(key, val);
    // }

    // let compile_status = cc_cmd.status().expect("Failed to compile lemon.c via CC");
    // assert!(compile_status.success(), "C compilation of lemon.c failed");

    // // 3. EXECUTION FIX: Run the tool, forcing the working directory to be 'c_src'
    // // This makes lemon naturally find 'lempar.c' and 'parse.y' right next to each other!
    // let lemon_status = Command::new(&lemon_exe_path)
    //     .current_dir("c_src") // <-- THE FIX: Moves execution context into c_src folder
    //     .arg("parse.y")       // Now it just looks for 'parse.y' in its current directory
    //     .status()
    //     .expect("Failed to run lemon binary");

    // assert!(lemon_status.success(), "Lemon failed to process parse.y");

    // generate_opcodes();

    // the C source is broken right now and wont compile :(


    // 2. Configure the C compiler engine
    // cc::Build::new()
    //     // Include the folder containing the SQLite header files
    //     .include("c_src")
    //     .include("src")
    //     // Add the specific SQLite subsystems you want to compile alongside Rust
    //     .file("c_src/parse.c")
    //     .file("c_src/vdbe.c")
    //     .file("c_src/pager.c")
    //     .file("c_src/btree.c")
    //     // SQLite relies heavily on SQLITE_OMIT flags to strip features.
    //     // For our Ninja port, we can aggressively strip out things we don't need!
    //     .define("SQLITE_OMIT_PARSER", None)
    //     .define("SQLITE_OMIT_AUTOVACUUM", None)
    //     // Enable high optimization flags for the C side
    //     .flag("-O3")
    //     // Compile everything into a static library called "libsqlite_core.a"
    //     .compile("sqlite_core");
}
