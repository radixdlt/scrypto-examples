from typing import Tuple
import subprocess
import os

def shell_command(*args: str) -> Tuple[str, str]:
    stdout, stderr = subprocess.Popen(
        args = " ".join(args), # type: ignore
        shell = True,
        stdout = subprocess.PIPE,
        stderr = subprocess.PIPE,
    ).communicate()

    return (str(stdout), str(stderr))

def script_dir() -> str:
    return os.path.dirname(os.path.realpath(__file__))

def main() -> None:
    script_directory: str = script_dir()

    for (root, _, files) in os.walk(script_directory):
        if 'Cargo.toml' in files:
            os.chdir(root)
            _formatting_std_out, _formatting_std_err = shell_command('scrypto', 'fmt')
            _check_std_out, check_std_err = shell_command('cargo', 'check', '--lib', '--tests', '--release')
            
            relative_path: str = root.replace(script_directory, '').strip('/\\')
            if 'Finished release' in check_std_err:
                print(f"\033[1;32m[✅] {relative_path}")
                print("[•] Cargo Check Succeeded")
            else:
                print(f"\033[1;31m[❌] {relative_path}")
                print("[•] Cargo Check Failed")
            print('')

if __name__ == "__main__":
    main()