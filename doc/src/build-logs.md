# logs

With make, you get the results of the build on the stdout. This is really difficult for reading, 
if you have a compile error, especially in C++ with templates, you may get hundreds of error lines.
If your build is multithreaded, this is even worse.

Yamake eases this process.
When building a node, yamake creates two files, one for stdout and one for stderr. They are placed in the sandbox directory.
the file `<sandbox>/make-report.yml` is generated at the end of each build, it provides informations about the build. You can get the path of the logs for a given node,
eg :

```sh
yq '.nodes[] | select(.pathbuf == "project_expand/main.o") | .stdout_path' sandbox/make-report.yml   
```
will return the path of the stdout captured when building this node.

You could use

```sh
for stderr in $(yq -r '.nodes[] | select (.status=="BuildFailed").stderr_path ' sandbox/make-report.yml) ; do
    echo $stderr
    if [[ $stderr  ]] ; then
        vim $stderr
    fi
done

```


