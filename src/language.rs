pub struct Language {
    pub code: &'static str,
    pub ext: &'static str,
    pub compile: &'static [&'static str],
    pub run: &'static str,
}

pub const LANGUAGES: &[Language] = &[
    Language {
        code: "rs",
        ext: "rs",
        compile: &["rustc --edition=2018 -O -o main main.rs"],
        run: "./main",
    },
    Language {
        code: "cpp",
        ext: "cc",
        compile: &["g++ -std=c++2a -o main -O3 main.cc"],
        run: "./main",
    },
    Language {
        code: "hs",
        ext: "hs",
        compile: &["ghc -fllvm -dynamic -o main main.hs"],
        run: "./main",
    },
    Language {
        code: "c",
        ext: "c",
        compile: &["gcc -std=c17 -o main -O3 main.c"],
        run: "./main",
    },
    Language {
        code: "py",
        ext: "py",
        compile: &["python3 -c 'import py_compile; py_compile.compile(\"main.py\")'"],
        run: "python3 main.py",
    },
    Language {
        code: "js",
        ext: "js",
        compile: &[],
        run: "node --max-old-space-size=2000 main.js",
    },
    Language {
        code: "sh",
        ext: "sh",
        compile: &["chmod +x main.sh"],
        run: "bash main.sh",
    },
    Language {
        code: "go",
        ext: "go",
        compile: &["go build main.go"],
        run: "./main",
    },
    Language {
        code: "java",
        ext: "java",
        compile: &["mv main.java Main.java", "javac Main.java"],
        run: "java -XX:MaxHeapSize=512m -XX:InitialHeapSize=512m -XX:CompressedClassSpaceSize=64m -XX:MaxMetaspaceSize=128m Main",
    },
];
