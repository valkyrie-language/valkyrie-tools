.class public Output
.super java/lang/Object

.method public <init>()V
  aload_0
  invokespecial java/lang/Object/<init>()V
  return
.end method

.method public static main()I
  .limit stack 100
  .limit locals 100
; Unknown New
  istore 0
  ldc N
  istore 1
  ldc N
  istore 2
  iload 0
  iload 1
  iload 2
; Unknown CallMethod
  istore 3
  ldc "Sum: "
  iload 3
  iadd
  invokestatic Output/print(I)I
  pop
  iload 3
  ldc N
  if_icmpgt L_gt_0
  iconst_0
  goto L_end_0
L_gt_0:
  iconst_1
L_end_0:
  ifeq L0
  ldc "Sum is greater than 20"
  invokestatic Output/print(I)I
  pop
  goto L1
L0:
  ldc "Sum is not greater than 20"
  invokestatic Output/print(I)I
  pop
L1:
  ldc 0
  istore 4
L2:
  iload 4
  ldc 5
  if_icmplt L_lt_1
  iconst_0
  goto L_end_1
L_lt_1:
  iconst_1
L_end_1:
  ifeq L3
  ldc "Loop: "
  iload 4
  iadd
  invokestatic Output/print(I)I
  pop
  iload 4
  iload 4
  ldc 1
  iadd
  pop
  goto L2
L3:
  iconst_0
  ireturn
.end method

.method public static main([Ljava/lang/String;)V
  .limit stack 100
  .limit locals 100
  return
.end method
