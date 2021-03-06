use jupyter_client::*;

#[tokio::main]
async fn main() {
    let client = JupyterClient::default();

    let resp = client.get_root_contents().await.unwrap();

    println!("{resp:?}");
    let resp = client.get_sessions().await.unwrap();
    println!("{resp:?}");

    let resp = client
        .put_contents("test.ipynb", vec!["12 + 24".to_string()].into())
        .await
        .unwrap();
    println!("{resp:?}");

    let kernelspecs = client.get_kernel_specs().await.unwrap();
    println!("kernelspecs :{kernelspecs:?}");

    let kernels = client.get_running_kernels().await.unwrap();
    println!("kernels:{kernels:?}");
    let resp = kernels.iter().find(|each| each.name == "rust");
    if resp.is_none() {
        let result = client
            .start_kernel(KernelPostRequest {
                name: "rust".to_string(),
                path: None,
            })
            .await
            .unwrap();
        println!("---{:?}", result);
    }

    let kernels = client.get_running_kernels().await.unwrap();

    let kernel = kernels.iter().find(|each| each.name == "rust").unwrap();
    let kernsl_cli = client.new_kernel_client(&kernel).unwrap();
    let resp = kernsl_cli.run_code(":dep tokio".into(), None).await;
    println!("{resp:?}");

    let resp = kernsl_cli.run_code("12 * 32".into(), None).await;
    println!("{resp:?}");

    let resp = kernsl_cli.run_code("a12 * 23".into(), None).await;
    println!("error: {resp:?}");

    let resp = kernsl_cli.run_code("".into(), None).await;
    println!("{resp:?}");

    let resp = kernsl_cli
        .run_code(
            r#"fn some(){

        println!("some")

    }"#
            .into(),
            None,
        )
        .await;
    println!("{resp:?}");

    let resp = kernsl_cli
        .run_code(r#" fn some(){println!("aaaa")} "#.into(), None)
        .await;
    println!("{resp:?}");
    let resp = kernsl_cli.run_code(r#" some();    "#.into(), None).await;
    println!("{resp:?}");
}
