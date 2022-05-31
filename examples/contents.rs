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

    let kernels = client.get_kernels().await.unwrap();
    println!("{kernels:?}");
    let resp = kernels.iter().find(|each| each.name == "rust");
    if resp.is_none() {
        client
            .start_kernel(KernelPostRequest {
                name: "rust".to_string(),
                path: None,
            })
            .await
            .unwrap();
    }

    let kernels = client.get_kernels().await.unwrap();
    let kernel = kernels.iter().find(|each| each.name == "rust").unwrap();
    let kernsl_cli = client.new_kernel_client(&kernel).unwrap();
    //let resp = kernsl_cli.run_code(":dep tokio".into(), None).await;
    //println!("{resp:?}");

    let resp = kernsl_cli.run_code("12 * 32".into(), None).await;
    println!("{resp:?}");

    let resp = kernsl_cli.run_code("a12 * 23".into(), None).await;
    println!("error: {resp:?}");
}
