"""Fixture-only regression checks for both owned staging helpers.

Set STAGE_TEST_TMPDIR to an approved existing temporary directory and optionally
STAGE_TEST_BASH to an existing Bash executable. No HAL artifact is used/executed.
Real symlink cases are explicitly skipped when the host cannot create symlinks;
mock canonicalization cases still test the escape checks on Windows.
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


PROJECT = Path(__file__).resolve().parents[1]
RPMH = PROJECT.parent / "oplus-rpmh-hal"


def shell_path(path):
    value = str(Path(path).absolute()).replace("\\", "/")
    if os.name == "nt" and len(value) >= 3 and value[1:3] == ":/":
        return "/" + value[0].lower() + value[2:]
    return value


class StageCases:
    @classmethod
    def setUpClass(cls):
        cls.bash = os.environ.get("STAGE_TEST_BASH") or shutil.which("bash")
        if not cls.bash or not Path(cls.bash).is_file():
            raise RuntimeError("set STAGE_TEST_BASH to an existing Bash executable")
        base = os.environ.get("STAGE_TEST_TMPDIR")
        if not base or not Path(base).is_dir():
            raise RuntimeError("STAGE_TEST_TMPDIR must be an approved existing directory")
        cls.temp_base = Path(base)
        if not cls.script.is_file():
            raise RuntimeError("both owned fork staging scripts must be present")

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="stage-safety-", dir=self.temp_base)
        self.addCleanup(temporary.cleanup)
        self.workspace = Path(temporary.name)
        self.root = self.workspace / "rom"
        self.root.mkdir()
        self.artifact = self.workspace / "mock-artifact"
        self.payload = b"MOCK FIXTURE ONLY - NOT A HAL EXECUTABLE\n"
        self.artifact.write_bytes(self.payload)
        self.fake_bin = self.workspace / "test-bin"
        self.fake_bin.mkdir()
        self.install_called = self.workspace / "install-called"

    def parent(self, partition="vendor"):
        parent = self.root / partition / "bin" / "hw"
        parent.mkdir(parents=True, exist_ok=True)
        return parent

    def stub(self, name, body):
        file = self.fake_bin / name
        file.write_text("#!/bin/bash\nset -eu\n" + body + "\n", encoding="ascii", newline="\n")
        file.chmod(0o755)

    def trap_install(self):
        # Even a regression must not copy through an escaping fixture link.
        self.stub("install", 'printf "called\\n" > "$STAGE_INSTALL_CALLED"\nexit 71')

    def run_stage(self, *, root=None, relative=None, artifact=None, mocked=False, extra=None):
        env = os.environ.copy()
        for key in ["BASH_ENV", "ENV", "ROM_TREE", "PROJECT_ARTIFACT", "RPMH_ARTIFACT",
                    "PROJECT_RELATIVE_EXECUTABLE", "RPMH_RELATIVE_EXECUTABLE"]:
            env.pop(key, None)
        env.update({
            "ROM_TREE": shell_path(root if root is not None else self.root),
            self.relative_key: relative if relative is not None else "vendor/bin/hw/" + self.name,
            self.artifact_key: shell_path(artifact if artifact is not None else self.artifact),
            "STAGE_INSTALL_CALLED": shell_path(self.install_called),
            "STAGE_TEST_PATH": (shell_path(self.fake_bin) + ":" if mocked else "") + "/usr/bin:/bin",
        })
        if extra:
            env.update(extra)
        return subprocess.run(
            # Git's Windows launcher may prepend its utilities to PATH. Set the
            # fixture PATH afterwards, then run the helper as a real script.
            [self.bash, "--noprofile", "--norc", "-c",
             'export PATH="$STAGE_TEST_PATH"; exec /bin/bash --noprofile --norc "$1"',
             "stage-fixture", shell_path(self.script)],
            cwd=self.workspace, env=env, capture_output=True, text=True, timeout=20,
        )

    def rejected(self, result, message=None):
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        if message:
            self.assertIn(message, result.stderr)
        self.assertFalse(self.install_called.exists(), "install must not be reached")

    def symlink(self, target, link, directory=False):
        try:
            os.symlink(target, link, target_is_directory=directory)
        except (OSError, NotImplementedError) as error:
            self.skipTest(f"actual symlink behavior untested on this host: {error}")
        if not link.is_symlink():
            self.skipTest("host did not create an actual symbolic link")

    def mock_resolver(self, root_input, root_output, parent_output):
        self.stub("realpath", '''last=${!#}
if [[ "$last" == "$TEST_ROOT_INPUT" ]]; then
    printf '%s\\n' "$TEST_ROOT_OUTPUT"
elif [[ "$last" == "$TEST_INPUT_PARENT" ]]; then
    printf '%s\\n' "$TEST_OUTPUT_PARENT"
else
    exit 72
fi''')
        return {
            "TEST_ROOT_INPUT": shell_path(root_input),
            "TEST_ROOT_OUTPUT": shell_path(root_output),
            "TEST_INPUT_PARENT": shell_path(root_output / "vendor/bin/hw"),
            "TEST_OUTPUT_PARENT": shell_path(parent_output),
        }

    def mocked_parent_escape(self, outside):
        self.parent()
        outside.mkdir(parents=True)
        self.trap_install()
        extra = self.mock_resolver(self.root, self.root, outside)
        result = self.run_stage(mocked=True, extra=extra)
        self.rejected(result, "escapes ROM root")
        self.assertEqual(list(outside.iterdir()), [])

    def test_regular_inside_copy(self):
        parent = self.parent()
        result = self.run_stage()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((parent / self.name).read_bytes(), self.payload)

    def test_odm_inside_copy(self):
        parent = self.parent("odm")
        result = self.run_stage(relative="odm/bin/hw/" + self.name)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((parent / self.name).read_bytes(), self.payload)

    def test_missing_root(self):
        self.trap_install()
        self.rejected(self.run_stage(root=self.workspace / "missing", mocked=True), "ROM root")

    def test_missing_parent(self):
        self.trap_install()
        self.rejected(self.run_stage(mocked=True), "deployment directory")

    def test_missing_artifact(self):
        self.parent()
        self.trap_install()
        self.rejected(self.run_stage(artifact=self.workspace / "missing", mocked=True), "missing artifact")

    def test_invalid_relative_paths(self):
        self.parent()
        self.trap_install()
        for relative in ["", "../" + self.name, "/vendor/bin/hw/" + self.name,
                         "vendor/bin/hw/../" + self.name, "vendor/bin/hw//" + self.name,
                         "vendor/bin/hw/unknown-service"]:
            with self.subTest(relative=relative):
                self.rejected(self.run_stage(relative=relative, mocked=True))

    def test_final_directory_is_rejected(self):
        (self.parent() / self.name).mkdir()
        self.trap_install()
        self.rejected(self.run_stage(mocked=True), "not a regular file")

    def test_mock_resolved_parent_escape(self):
        self.mocked_parent_escape(self.workspace / "outside")

    def test_mock_install_trap_is_active(self):
        parent = self.parent()
        self.trap_install()
        result = self.run_stage(mocked=True)
        self.assertEqual(result.returncode, 71, result.stdout + result.stderr)
        self.assertTrue(self.install_called.exists())
        self.assertFalse((parent / self.name).exists())

    def test_mock_root_prefix_lookalike_escape(self):
        self.mocked_parent_escape(self.workspace / "rom-other" / "hw")

    def test_mock_explicit_root_alias_is_allowed(self):
        parent = self.parent()
        selected_root = self.workspace / "selected-root"
        selected_root.mkdir()
        extra = self.mock_resolver(selected_root, self.root, parent)
        result = self.run_stage(root=selected_root, mocked=True, extra=extra)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((parent / self.name).read_bytes(), self.payload)
        self.assertEqual(list(selected_root.iterdir()), [])

    def test_mock_internal_parent_alias_stays_inside_root(self):
        self.parent()
        physical_parent = self.root / "physical-hw"
        physical_parent.mkdir()
        extra = self.mock_resolver(self.root, self.root, physical_parent)
        result = self.run_stage(mocked=True, extra=extra)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((physical_parent / self.name).read_bytes(), self.payload)

    def test_readlink_fallback(self):
        parent = self.parent()
        self.stub("realpath", "exit 1")
        result = self.run_stage(mocked=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((parent / self.name).read_bytes(), self.payload)

    def test_actual_parent_symlink_escape(self):
        parent = self.root / "vendor/bin"
        parent.mkdir(parents=True)
        outside = self.workspace / "outside"
        outside.mkdir()
        self.symlink(outside, parent / "hw", directory=True)
        self.trap_install()
        self.rejected(self.run_stage(mocked=True), "escapes ROM root")
        self.assertEqual(list(outside.iterdir()), [])

    def test_actual_final_symlink(self):
        destination = self.parent() / self.name
        outside = self.workspace / "outside-target"
        outside.write_bytes(b"UNCHANGED MOCK SENTINEL")
        self.symlink(outside, destination)
        self.trap_install()
        self.rejected(self.run_stage(mocked=True), "destination is a symlink")
        self.assertEqual(outside.read_bytes(), b"UNCHANGED MOCK SENTINEL")

    def test_actual_dangling_final_symlink(self):
        destination = self.parent() / self.name
        outside = self.workspace / "missing-target"
        self.symlink(outside, destination)
        self.trap_install()
        self.rejected(self.run_stage(mocked=True), "destination is a symlink")
        self.assertFalse(outside.exists())

    def test_actual_explicit_root_symlink_is_allowed(self):
        parent = self.parent()
        selected_root = self.workspace / "selected-root"
        self.symlink(self.root, selected_root, directory=True)
        result = self.run_stage(root=selected_root)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((parent / self.name).read_bytes(), self.payload)

    def test_actual_internal_symlink_stays_inside_root(self):
        parent = self.root / "vendor/bin"
        parent.mkdir(parents=True)
        physical_parent = self.root / "physical-hw"
        physical_parent.mkdir()
        self.symlink(physical_parent, parent / "hw", directory=True)
        result = self.run_stage()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual((physical_parent / self.name).read_bytes(), self.payload)

    def test_final_symlink_guard_precedes_install(self):
        source = self.script.read_text(encoding="utf-8")
        guard = source.index('[[ ! -L "$DEST" ]]')
        self.assertLess(source.index('DEST="$DEST_PARENT/'), guard)
        self.assertLess(guard, source.index('install -m 0755 -- "$SRC" "$DEST"'))


class ProjectStageTests(StageCases, unittest.TestCase):
    script = PROJECT / "scripts/stage_vendor.sh"
    name = "vendor.oplus.hardware.stability.oplus_project-V3-service"
    relative_key = "PROJECT_RELATIVE_EXECUTABLE"
    artifact_key = "PROJECT_ARTIFACT"


class RpmhStageTests(StageCases, unittest.TestCase):
    script = RPMH / "scripts/stage_vendor.sh"
    name = "vendor.oplus.hardware.rpmh-V1-service"
    relative_key = "RPMH_RELATIVE_EXECUTABLE"
    artifact_key = "RPMH_ARTIFACT"


if __name__ == "__main__":
    unittest.main(verbosity=2)
