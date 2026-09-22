import test from "node:test";
import assert from "node:assert/strict";
import {
  Project,
  Note,
  BuildError,
  defaultInspectLimits,
} from "../dist/index.mjs";

test("all inspection budgets accept exact u64 bigint inputs for build and diff", async () => {
  const defaults = defaultInspectLimits();
  assert.equal(Object.keys(defaults).length, 11);
  for (const value of Object.values(defaults))
    assert.ok(Number.isSafeInteger(value));
  const project = new Project("Budgets");
  project.addNote(Note.basic("one", "1"));
  const limits = Object.fromEntries(
    Object.keys(defaults).map((key) => [key, 18446744073709551615n]),
  );
  limits.maxArchiveBytes = 9007199254740993n;
  const report = await project.build({ inspectLimits: limits });
  try {
    report.ensureSuccess();
    for (const value of [
      Number.MAX_SAFE_INTEGER,
      9007199254740992n,
      9007199254740993n,
      18446744073709551615n,
    ]) {
      const exact = await project.build({
        inspectLimits: { maxArchiveBytes: value },
      });
      exact.ensureSuccess();
      await exact.artifactHandle.close();
    }
    const diff = await project.diffAgainstApkg(report.artifact.path, {
      inspectLimits: limits,
    });
    assert.ok(diff);
    await assert.rejects(
      project.build({ inspectLimits: { maxArchiveBytes: 0n } }),
      (error) =>
        error instanceof BuildError &&
        error.report.diagnosticCodes.includes(
          "INSPECT.RESOURCE_LIMIT_EXCEEDED",
        ),
    );
    for (const value of [
      -1n,
      18446744073709551616n,
      -1,
      1.5,
      NaN,
      Infinity,
      9007199254740992,
      "12",
      null,
    ]) {
      await assert.rejects(
        project.build({ inspectLimits: { maxEntries: value } }),
        TypeError,
      );
      await assert.rejects(
        project.diffAgainstApkg(report.artifact.path, {
          inspectLimits: { maxEntries: value },
        }),
        TypeError,
      );
    }
    await assert.rejects(
      project.build({ inspectLimits: { unknown: 10n } }),
      TypeError,
    );
    assert.equal(typeof BigInt.prototype.toJSON, "undefined");
  } finally {
    await report.artifactHandle.close();
  }
});
