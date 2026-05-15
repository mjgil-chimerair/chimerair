module {
  func.func @test_identity_i32(i32, i32) -> i32 {
    ^bb0(%arg0: i32, %arg1: i32):
      func.return %arg0 : i32
  }

  func.func @test_identity_i64(i64, i64) -> i64 {
    ^bb0(%arg0: i64, %arg1: i64):
      func.return %arg0 : i64
  }

  func.func @test_add_i32(i32, i32) -> i32 {
    ^bb0(%arg0: i32, %arg1: i32):
      %0 = arith.addi %arg0, %arg1 : i32
      func.return %0 : i32
  }

  func.func @test_call_add(i32, i32) -> i32 {
    ^bb0(%arg0: i32, %arg1: i32):
      %0 = func.call @test_add_i32(%arg0, %arg1) : (i32, i32) -> i32
      func.return %0 : i32
  }

  func.func @test_no_args() -> i32 {
    ^bb0:
      %0 = arith.constant 42 : i32
      func.return %0 : i32
  }

  func.func private @nested_func(i32) -> i32

  func.func @test_call_nested(i32) -> i32 {
    ^bb0(%arg0: i32):
      %0 = func.call @nested_func(%arg0) : (i32) -> i32
      func.return %0 : i32
  }
}
