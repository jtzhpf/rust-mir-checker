# This script runs examples under various options
# This is mainly used to gather coverage information

import os
import sys
import subprocess
import time
import re

unit_tests_list = [
    {"name": "alloc-test", "entry": "main"},
    {"name": "annotation", "entry": "main"},
    {"name": "arith", "entry": "main"},
    {"name": "array", "entry": "main"},
    {"name": "assignment", "entry": "main"},
    {"name": "big-loop", "entry": "main"},
    {"name": "cast", "entry": "main"},
    {"name": "crate-bin-test", "entry": "main"},
    {"name": "crate-lib-test", "entry": "foo"},
    {"name": "empty", "entry": "main"},
    {"name": "enum-test", "entry": "main"},
    {"name": "function-call", "entry": "main"},
    {"name": "index", "entry": "main"},
    {"name": "iterator", "entry": "main"},
    {"name": "loop-test", "entry": "main"},
    {"name": "negation", "entry": "main"},
    {"name": "recursion", "entry": "main"},
    {"name": "size-of", "entry": "main"},
    {"name": "struct-test", "entry": "main"},
    {"name": "vector", "entry": "main"},
    {"name": "widen-narrow", "entry": "main"},
]

safe_bugs_list = [
    {"name": "division-by-zero", "entry": "main"},
    {"name": "incorrect-boundary-check", "entry": "main"},
    {"name": "incorrect-cast", "entry": "main"},
    {"name": "integer-overflow", "entry": "main"},
    {"name": "out-of-bound-index", "entry": "main"},
    {"name": "unreachable", "entry": "main"},
]

unsafe_bugs_list = [
    {"name": "double-free-and-use-after-free", "entry": "main"},
    {"name": "double-free", "entry": "main"},
    {"name": "offset", "entry": "main"},
    {"name": "use-after-free(CVE-2019-15551)", "entry": "main"},
    {"name": "use-after-free(CVE-2019-16140)", "entry": "main"},
]

abstract_domains = ["interval", "octagon", "polyhedra", "linear_equalities", "ppl_polyhedra",
                    "ppl_linear_congruences", "pkgrid_polyhedra_linear_congruences"]

executable = os.path.abspath("../target/debug/cargo-mir-checker")


def run_test(test_list, test_dir, allow_error, file):
    # for (test, domain_type) in itertools.product(test_list, abstract_domains):
    for test in test_list:
        ok = False
        for domain_type in abstract_domains:
            # First run cargo clean to make sure that cargo does not use cache
            p = subprocess.Popen(["cargo", "clean"], cwd=os.path.join(test_dir, test["name"]))
            p.wait()

            my_env = os.environ.copy()
            # Disabling logging will save a lot of execution time!
            # my_env["RUST_LOG"] = "rust_mir_checker"

            # Customized options
            
            
            if file:
                p = subprocess.Popen([executable, "mir-checker", "--", "--domain", domain_type, "--entry", test["entry"], "--widening_delay", "5",
                                    "--narrowing_iteration", "5", "--deny_warnings"], cwd=os.path.join(test_dir, test["name"]), env=my_env,stderr=subprocess.PIPE)
                _, stderr_output = p.communicate()
                #print(stderr_output.decode("utf-8"))  # 输出标准输出到屏幕
                with open(file, 'a') as f:
                    f.write(stderr_output.decode())  # Write stderr output to file
            else:
                p = subprocess.Popen([executable, "mir-checker", "--", "--domain", domain_type, "--entry", test["entry"], "--widening_delay", "5",
                                    "--narrowing_iteration", "5", "--deny_warnings"], cwd=os.path.join(test_dir, test["name"]), env=my_env)
                p.communicate()[0]
            rc = p.returncode
            if rc == 0:
                ok = True

        if not ok and not allow_error:
            raise Exception("All abstract domains cannot reason about the verification conditions for \"{}\"".format(test["name"]))


def exec_run_test():
    # Run tests
    try:
        start = time.time()
        run_test(unit_tests_list, "unit-tests", True, None)
        run_test(safe_bugs_list, "safe-bugs", True, None)
        run_test(unsafe_bugs_list, "unsafe-bugs", True, None)
        end = time.time()
        print("All tests are passed! Elapsed time:", end - start)
    except Exception as e:
        print(e)
        sys.exit(1)  # This error code will cause a failure in CI service, so we know there are some problems


def exec_run_test_output(output_file):
    # 检查文件是否存在
    if os.path.exists(output_file):
        # 删除文件
        os.remove(output_file)
        
    # Redirect stderr to a file
    try:
        start = time.time()
        run_test(unit_tests_list, "unit-tests", True, output_file)
        run_test(safe_bugs_list, "safe-bugs", True, output_file)
        run_test(unsafe_bugs_list, "unsafe-bugs", True, output_file)
        end = time.time()
        print("All tests are passed! Elapsed time:", end - start)
    except Exception as e:
        print(e)
        sys.exit(1)  # This error code will cause a failure in CI service, so we know there are some problems


def filter_time_info(line):
    # 正则表达式模式，匹配时间信息
    time_pattern = r'Checking\s\S+\sv\d+\.\d+\.\d+\s\((?:\/\S+)+\)|target\(s\)\sin\s\d+\.\d+s'
    # 使用正则表达式替换时间信息为空字符串
    return re.sub(time_pattern, '', line)

def compare_files(file1_path, file2_path):
    # 读取文件内容并过滤时间信息
    with open(file1_path, 'r') as file1, open(file2_path, 'r') as file2:
        lines1 = [filter_time_info(line) for line in file1]
        lines2 = [filter_time_info(line) for line in file2]

    # 比较过滤后的内容
    return lines1 == lines2


def check(ref_file, test_file):
    # Run check
    try:
        if compare_files(ref_file, test_file):
            print("Files are identical (excluding time information).")
        else:
            print("Files are different.\n")
            # 调用 diff 命令,并捕获输出
            diff_process = subprocess.Popen(['diff', '--color=always', ref_file, test_file], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            output, error = diff_process.communicate()

            # 输出结果
            print(output.decode('utf-8'))
            
            sys.exit(1)
        
    except Exception as e:
        print(e)
        sys.exit(1)  # This error code will cause a failure in CI service, so we know there are some problems

def generate_ref_file(ref_file):
    exec_run_test_output(ref_file)

# main
try:
    #print(sys.argv)
    if len(sys.argv) == 1 or (len(sys.argv) == 2 and sys.argv[1] == "run"):
        exec_run_test()
    elif len(sys.argv) == 2 and sys.argv[1] == "check":
        output_file = "test.results"
        ref_file = "ref.results"
        exec_run_test_output(output_file)
        check(ref_file, output_file)
        os.remove(output_file)
    elif len(sys.argv) == 2 and sys.argv[1] == "gen":
        ref_file = "ref.results"
        generate_ref_file(ref_file)
    else:
        sys.exit(1)
except Exception as e:
    print(e)
    sys.exit(1)  # This error code will cause a failure in CI service, so we know there are some problems
