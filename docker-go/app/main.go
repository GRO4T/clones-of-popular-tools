package main

import (
	"os"
	"os/exec"
	"strconv"
	"strings"
  "syscall"
)

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
func main() {
	command := os.Args[3]
	args := os.Args[4:len(os.Args)]

	cmd := exec.Command(command, args...)
  
  cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

  if err := syscall.Chroot("./chroot-jail"); err != nil {
			panic(err)
	}

  cmd.SysProcAttr = &syscall.SysProcAttr{
		Cloneflags: syscall.CLONE_NEWUTS | syscall.CLONE_NEWPID,
	}

  err := cmd.Run()

	if err != nil {
    e := strings.Split(err.Error(), " ")
		errNum, _ := strconv.Atoi(e[len(e)-1])
		os.Exit(errNum)
	}
}
