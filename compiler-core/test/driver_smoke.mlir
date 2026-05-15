module {
  func.func private @ffi_import(i32) -> i32

  func.func @test(%arg0: i32) -> i32 {
    %0 = func.call @ffi_import(%arg0) : (i32) -> i32
    return %0 : i32
  }
}
