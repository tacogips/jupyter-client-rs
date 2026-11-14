
rust client for jupyter lab

just PoC now

## usage
see [./examples](./examples)

The jupyter lab process should starts without token, passowrd and the checking xsrf disabled
```
jupyter lab --NotebookApp.token='' --NotebookApp.password='' --NotebookApp.disable_check_xsrf=True
```

the suggeting docker image: https://github.com/tacogips/jupyter-lab-rust

