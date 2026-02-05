import LeanVerifier.Verifier

def main : IO Unit := do
  IO.println "=== Lean Verifier Test ==="

  let value : UInt64 := 42
  let salt : UInt64 := 12345

  let commitment := createCommitment value salt
  IO.println s!"Commitment de {value} con salt {salt}: {commitment}"

  let valid := verifyCommitment commitment value salt
  IO.println s!"Verificación: {valid}"

  let invalid := verifyCommitment commitment 99 salt
  IO.println s!"Verificación con valor incorrecto: {invalid}"
