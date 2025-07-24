@.str.1 = private unnamed_addr constant [14 x i8] c"10 + 3 = %lld\00", align 1
@.str.5 = private unnamed_addr constant [14 x i8] c"10 - 3 = %lld\00", align 1
@.str.9 = private unnamed_addr constant [14 x i8] c"10 * 3 = %lld\00", align 1
@.str.13 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.15 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.18 = private unnamed_addr constant [13 x i8] c"10 > 3 is %s\00", align 1
@.str.22 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.24 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.27 = private unnamed_addr constant [13 x i8] c"10 < 3 is %s\00", align 1
@.str.31 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.33 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.36 = private unnamed_addr constant [15 x i8] c"10 >= 10 is %s\00", align 1
@.str.40 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.42 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.45 = private unnamed_addr constant [15 x i8] c"10 <= 10 is %s\00", align 1
@.str.49 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.51 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.54 = private unnamed_addr constant [15 x i8] c"10 == 10 is %s\00", align 1
@.str.58 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.60 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.63 = private unnamed_addr constant [14 x i8] c"10 != 3 is %s\00", align 1
@.str.67 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.69 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.72 = private unnamed_addr constant [21 x i8] c"true and false is %s\00", align 1
@.str.76 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.78 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.81 = private unnamed_addr constant [20 x i8] c"true or false is %s\00", align 1
@.str.85 = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.87 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.90 = private unnamed_addr constant [15 x i8] c"not true is %s\00", align 1
@.str.94 = private unnamed_addr constant [15 x i8] c"neg 10 is %lld\00", align 1
target triple = "x86_64-unknown-linux-gnu"
%coral.list = type { i8*, i64, i64 }
%coral.map = type { i8*, i64, i64 }
%coral.result = type { i8*, i8 }

declare i32 @printf(i8*, ...)
declare i32 @puts(i8*)
declare i8* @malloc(i64)
declare i8* @realloc(i8*, i64)
declare i8* @map_create(i64, i64)
declare void @map_insert(i8*, i8*, i8*)
declare i8* @get(i8*)
declare %coral.list* @glob_glob(i8*)

declare i8* @pipe_create()
declare void @pipe_connect_source(i8*, i8*, i8*)
declare void @pipe_connect_destination(i8*, i8*, i8*)
declare i32 @pipe_write(i8*, i8*)
declare i8* @pipe_read(i8*)

declare i8* @io_read(i8*)
declare i32 @io_write(i8*, i8*)
declare void @io_print(i8*)

declare i8* @actor_create(i32, i8*)
declare void @actor_send(i8*, i32, i8*)

declare i8* @store_get_by_id(i8*, i64)

declare i8* @value_to_string(i64)
define void @log(i8* %message) {
entry:
  ret void
  ret void
}
define i32 @main() {
entry:
  %0 = add i64 10, 3
  %2 = getelementptr inbounds [15 x i8], [15 x i8]* @.str.1, i64 0, i64 0
  %3 = call i32 (i8*, ...) @printf(i8* %2, i64 %0)
  %4 = sub i64 10, 3
  %6 = getelementptr inbounds [15 x i8], [15 x i8]* @.str.5, i64 0, i64 0
  %7 = call i32 (i8*, ...) @printf(i8* %6, i64 %4)
  %8 = mul i64 10, 3
  %10 = getelementptr inbounds [15 x i8], [15 x i8]* @.str.9, i64 0, i64 0
  %11 = call i32 (i8*, ...) @printf(i8* %10, i64 %8)
  %12 = icmp sgt i64 10, 3
  %14 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.13, i64 0, i64 0
  %16 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.15, i64 0, i64 0
  %17 = select i1 %12, i8* %14, i8* %16
  %19 = getelementptr inbounds [14 x i8], [14 x i8]* @.str.18, i64 0, i64 0
  %20 = call i32 (i8*, ...) @printf(i8* %19, i8* %17)
  %21 = icmp slt i64 10, 3
  %23 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.22, i64 0, i64 0
  %25 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.24, i64 0, i64 0
  %26 = select i1 %21, i8* %23, i8* %25
  %28 = getelementptr inbounds [14 x i8], [14 x i8]* @.str.27, i64 0, i64 0
  %29 = call i32 (i8*, ...) @printf(i8* %28, i8* %26)
  %30 = icmp sge i64 10, 10
  %32 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.31, i64 0, i64 0
  %34 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.33, i64 0, i64 0
  %35 = select i1 %30, i8* %32, i8* %34
  %37 = getelementptr inbounds [16 x i8], [16 x i8]* @.str.36, i64 0, i64 0
  %38 = call i32 (i8*, ...) @printf(i8* %37, i8* %35)
  %39 = icmp sle i64 10, 10
  %41 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.40, i64 0, i64 0
  %43 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.42, i64 0, i64 0
  %44 = select i1 %39, i8* %41, i8* %43
  %46 = getelementptr inbounds [16 x i8], [16 x i8]* @.str.45, i64 0, i64 0
  %47 = call i32 (i8*, ...) @printf(i8* %46, i8* %44)
  %48 = icmp eq i64 10, 10
  %50 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.49, i64 0, i64 0
  %52 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.51, i64 0, i64 0
  %53 = select i1 %48, i8* %50, i8* %52
  %55 = getelementptr inbounds [16 x i8], [16 x i8]* @.str.54, i64 0, i64 0
  %56 = call i32 (i8*, ...) @printf(i8* %55, i8* %53)
  %57 = icmp ne i64 10, 3
  %59 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.58, i64 0, i64 0
  %61 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.60, i64 0, i64 0
  %62 = select i1 %57, i8* %59, i8* %61
  %64 = getelementptr inbounds [15 x i8], [15 x i8]* @.str.63, i64 0, i64 0
  %65 = call i32 (i8*, ...) @printf(i8* %64, i8* %62)
  %66 = and i1 true, false
  %68 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.67, i64 0, i64 0
  %70 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.69, i64 0, i64 0
  %71 = select i1 %66, i8* %68, i8* %70
  %73 = getelementptr inbounds [22 x i8], [22 x i8]* @.str.72, i64 0, i64 0
  %74 = call i32 (i8*, ...) @printf(i8* %73, i8* %71)
  %75 = or i1 true, false
  %77 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.76, i64 0, i64 0
  %79 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.78, i64 0, i64 0
  %80 = select i1 %75, i8* %77, i8* %79
  %82 = getelementptr inbounds [21 x i8], [21 x i8]* @.str.81, i64 0, i64 0
  %83 = call i32 (i8*, ...) @printf(i8* %82, i8* %80)
  %84 = xor i1 true, true
  %86 = getelementptr inbounds [6 x i8], [6 x i8]* @.str.85, i64 0, i64 0
  %88 = getelementptr inbounds [7 x i8], [7 x i8]* @.str.87, i64 0, i64 0
  %89 = select i1 %84, i8* %86, i8* %88
  %91 = getelementptr inbounds [16 x i8], [16 x i8]* @.str.90, i64 0, i64 0
  %92 = call i32 (i8*, ...) @printf(i8* %91, i8* %89)
  %93 = sub i64 0, 10
  %95 = getelementptr inbounds [16 x i8], [16 x i8]* @.str.94, i64 0, i64 0
  %96 = call i32 (i8*, ...) @printf(i8* %95, i64 %93)
  ret i32 0
}
