mod support;

use std::fs;

fn oc_iter_content() -> String {
    let path = support::repo_root().join("scripts/oc-iter.ps1");
    fs::read_to_string(&path).expect("scripts/oc-iter.ps1 deve existir")
}

fn parametro_inteiro(script: &str, nome: &str) -> i64 {
    let agulha = format!("[int]${nome} = ");
    let pos = script
        .find(&agulha)
        .unwrap_or_else(|| panic!("oc-iter.ps1 deve declarar o parametro [int]${nome}"));
    let resto = &script[pos + agulha.len()..];
    let fim = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    resto[..fim]
        .parse()
        .unwrap_or_else(|_| panic!("valor de ${nome} deve ser inteiro"))
}

fn corpo_da_funcao(script: &str, inicio: usize) -> &str {
    let resto = &script[inicio..];
    match resto.find(
        "
}
",
    ) {
        Some(fim) => &resto[..fim],
        None => resto,
    }
}

#[test]
fn espera_da_rodada_nao_e_um_waitforexit_cego_de_parede_inteira() {
    let script = oc_iter_content();

    assert!(
        !script.contains("WaitForExit($TimeoutMin"),
        "a espera nao pode ser um unico `WaitForExit($TimeoutMin * 60 * 1000)`: bloqueado assim, \
         o loop nao consegue distinguir rodada lenta de rodada travada e paga a parede inteira \
         pelas duas. Medido em 30/07: duas rodadas morreram com 89 s e 100 s de vida ativa e \
         seguraram o loop por ~43 min cada uma sem emitir um unico evento."
    );
    assert!(
        script.contains("HasExited"),
        "a espera deve ser um laco que observa o processo (`HasExited`), nao uma chamada cega"
    );
}

#[test]
fn espera_observa_o_crescimento_do_json_da_rodada() {
    let script = oc_iter_content();

    assert!(
        script.contains("(Get-Item $outFile).Length"),
        "o sinal de vida da rodada e o JSON de saida crescendo: a espera tem de LER \
         `(Get-Item $outFile).Length` a cada volta. Sem ler o tamanho, nao ha como saber que o \
         provedor parou de responder."
    );
    let pos_len = script
        .find("(Get-Item $outFile).Length")
        .expect("leitura do tamanho ja verificada acima");
    let pos_cmp = script
        .find("$ultimoAvanco).TotalMinutes")
        .expect("a janela de travamento tem de ser medida a partir de `$ultimoAvanco`");
    assert!(
        pos_len < pos_cmp,
        "o tamanho tem de ser lido ANTES de a janela ser avaliada (posicoes {pos_len} e {pos_cmp})"
    );
    let janela = &script[pos_len..pos_cmp];
    assert!(
        janela.contains("$ultimoAvanco = Get-Date"),
        "entre ler o tamanho e avaliar a janela, a marca de avanco tem de ser REATRIBUIDA \
         (`$ultimoAvanco = Get-Date`) quando o tamanho muda. Sem essa reatribuicao a marca \
         congela na largada e o detector mata TODA rodada ao fim da janela, inclusive as que \
         estao trabalhando — foi o mutante m6 da bateria 0098, que sobreviveu a primeira versao \
         deste teste."
    );
}

#[test]
fn travamento_tem_rotulo_proprio_distinto_de_timeout() {
    let script = oc_iter_content();

    assert!(
        script.contains("falha:travamento"),
        "rodada morta por travamento tem de receber rotulo PROPRIO (`falha:travamento`) na linha \
         de metrica. Sem rotulo distinto, `docs/metricas.csv` mistura 45 min de trabalho lento \
         com 90 s de provedor mudo, e as duas exigem remedios opostos."
    );
    assert!(
        script.contains("falha:timeout"),
        "o rotulo `falha:timeout` deve continuar existindo para a rodada que trabalhou ate a parede"
    );
}

#[test]
fn janela_de_travamento_e_menor_que_a_parede_da_rodada() {
    let script = oc_iter_content();
    let travamento = parametro_inteiro(&script, "TravamentoMin");
    let parede = parametro_inteiro(&script, "TimeoutMin");

    assert!(
        travamento > 0,
        "a janela de travamento tem de ser positiva (valor atual: {travamento})"
    );
    assert!(
        travamento < parede,
        "a janela de travamento ({travamento} min) tem de ser MENOR que a parede da rodada \
         ({parede} min). Igual ou maior, o detector nunca dispara antes do timeout e o conserto \
         inteiro vira decoracao."
    );
}

#[test]
fn rodada_morta_encerra_a_sessao_no_daemon_e_nao_so_o_cliente() {
    let script = oc_iter_content();

    let pos_funcao = script.find("function Stop-SessaoDoTrabalhador").expect(
        "oc-iter.ps1 deve ter uma funcao que encerra a SESSAO, nao so o processo cliente. \
         `Stop-Process` no `opencode run --attach` mata o cliente; a sessao vive dentro do \
         daemon `opencode serve` e continua escrevendo na arvore. Medido em 30/07: depois de a \
         rodada ser declarada morta, ela commitou 3x direto na main local, resetou HEAD e \
         renomeou branch, por ~3 min.",
    );
    let corpo = corpo_da_funcao(&script, pos_funcao);
    assert!(
        corpo.contains("Get-Process") && corpo.contains("opencode"),
        "a funcao tem de alcancar o processo do DAEMON pelo nome (`Get-Process ... opencode`); \
         matar so `$p.Id` e o que ja se fazia e nao resolve. Conferido dentro do CORPO da          funcao: procurar no resto do arquivo deixa passar, porque `opencode` aparece adiante."
    );
}

#[test]
fn encerramento_da_sessao_esta_no_caminho_de_falha() {
    let script = oc_iter_content();

    let pos_guarda = script
        .find("$resultado -like \"falha:")
        .expect("o encerramento da sessao tem de ser disparado por uma guarda de FALHA generica");
    let depois = &script[pos_guarda..];
    let pos_chamada = depois.find("Stop-SessaoDoTrabalhador").unwrap_or_else(|| {
        panic!(
            "dentro da guarda de falha tem de haver a chamada a Stop-SessaoDoTrabalhador. \
             Amarrar o encerramento a UM rotulo so deixa o outro modo de morte com sessao viva."
        )
    });
    let entre = &depois[..pos_chamada];
    assert!(
        entre.len() < 400,
        "a chamada deve estar DENTRO do bloco da guarda de falha, nao centenas de caracteres \
         adiante (distancia atual: {} chars)",
        entre.len()
    );
}

#[test]
fn apos_matar_o_daemon_espera_a_porta_liberar() {
    let script = oc_iter_content();
    let pos_funcao = script
        .find("function Stop-SessaoDoTrabalhador")
        .expect("funcao de encerramento deve existir");
    let corpo = corpo_da_funcao(&script, pos_funcao);
    let pos_kill = corpo
        .find("Stop-Process")
        .expect("a funcao deve matar o processo do daemon");
    let depois_do_kill = &corpo[pos_kill..];

    assert!(
        depois_do_kill.contains("Test-Connection"),
        "depois de matar o daemon a funcao tem de ESPERAR a porta liberar (`Test-Connection`): \
         a proxima rodada faz `--attach` na porta e, se o daemon ainda estiver morrendo, ela \
         anexa num processo que vai sumir — troca uma corrida por outra"
    );
}

#[test]
fn deteccao_da_porta_usa_o_endereco_em_que_o_daemon_escuta() {
    let script = oc_iter_content();

    assert!(
        !script.contains("-TargetName localhost -TcpPort"),
        "`Test-Connection -TargetName localhost` resolve para ::1 (IPv6), e o `opencode serve` \
         escuta em 127.0.0.1 (IPv4): medido em 30/07 com o daemon vivo, `localhost` responde \
         False e `127.0.0.1` responde True. Com `localhost` a checagem de subida acha que o \
         daemon esta morto em TODA rodada, e a espera pela porta liberar nunca espera."
    );
    assert!(
        script.contains("-TargetName 127.0.0.1 -TcpPort"),
        "a checagem da porta tem de existir usando 127.0.0.1"
    );
}

fn linha_de(script: &str, agulha: &str) -> String {
    let pos = script
        .find(agulha)
        .unwrap_or_else(|| panic!("o script deve conter `{agulha}`"));
    script[..pos].rsplit('\n').next().unwrap_or("").to_string()
}

#[test]
fn descoberta_do_binario_nao_e_so_o_caminho_do_npm_do_windows() {
    let script = oc_iter_content();
    let inicio = script
        .find("Get-Command opencode")
        .expect("o script tem de descobrir o binario por Get-Command");
    let fim = script[inicio..]
        .find("Write-Error")
        .map(|p| p + inicio)
        .expect("a descoberta tem de terminar numa guarda de 'nao encontrado'");
    let trecho = &script[inicio..fim];

    assert!(
        trecho.contains("$IsWindows"),
        "a descoberta do binario e a UNICA coisa do script que depende de sistema. No Windows o \
         caminho que o `Get-Command` devolve e um shim do npm — o .ps1 nao roda via Start-Process \
         e o .cmd degrada aspas (smoke 0008b/1 e /2) — e o binario real mora sob `node_modules`. \
         Fora do Windows a instalacao e nativa e esse caminho JA e o binario. Sem bifurcar, o \
         script monta um `...\\node_modules\\opencode-ai\\bin\\opencode.exe` que nao existe no \
         Linux e aborta no Write-Error antes de gastar um token — a rodada morre sem sequer \
         aparecer na metricas.csv"
    );
    assert!(
        trecho.contains("\n    $shim\n"),
        "o braco de fora do Windows tem de resolver para o proprio caminho devolvido pelo \
         Get-Command ($shim). Bifurcar por plataforma e montar caminho de npm nos dois lados \
         nao conserta nada"
    );
    let atribuicoes = trecho.matches("$oc = ").count();
    assert_eq!(
        atribuicoes, 1,
        "$oc tem de ser atribuido UMA unica vez na descoberta (encontradas {atribuicoes}). As duas \
         afirmacoes acima olham a FORMA do texto, nao o VALOR que sobra em $oc: com uma segunda \
         atribuicao logo abaixo do if — `$oc = Join-Path ... node_modules ...` — a bifurcacao \
         continua escrita, os dois asserts passam e o script volta a apontar para um caminho que \
         nao existe no Linux. Achado da revisao adversarial da 0139 (revisor gpt-5.6-luna), \
         confirmado por mutante que sobreviveu 13/13 antes desta terceira afirmacao. Mesma familia \
         dos mutantes sobreviventes de 0094, 0098, 0100 e 0101: teste que le o texto em vez de \
         cobrar o efeito"
    );
}

#[test]
fn flag_de_janela_do_windows_nao_vaza_para_o_daemon_no_linux() {
    let script = oc_iter_content();

    assert!(
        linha_de(&script, "$serveArgs[\"WindowStyle\"]").contains("$IsWindows"),
        "medido em 01/08 nesta maquina: `Start-Process -WindowStyle Hidden` LANCA no PowerShell \
         do Linux (\"The parameter '-WindowStyle' is not supported for the cmdlet \
         'Start-Process' on this edition of PowerShell\"). Com $ErrorActionPreference = \"Stop\" \
         isso mata a rodada na subida do daemon, antes do primeiro token. A guarda tem de estar \
         na MESMA linha da atribuicao: conferida no arquivo inteiro, $IsWindows aparece adiante \
         por outro motivo e o teste fica verde com a guarda apagada"
    );
    assert!(
        !script.contains("-WindowStyle"),
        "esconder janela e conceito de Windows e nao pode ser emitido como parametro fixo da \
         chamada: passe por splat, so quando $IsWindows"
    );
}

#[test]
fn modelo_padrao_e_o_trabalhador_em_vigor() {
    let script = oc_iter_content();

    assert!(
        script.contains("[string]$Model = \"opencode-go/gpt-5.6-luna\""),
        "o default de -Model e o unico lugar que decide quem trabalha quando ninguem passa a \
         flag, e foi assim que as iteracoes 0009..0017 rodaram na geracao anterior do modelo sem \
         ninguem ter decidido isso (docs/orquestracao.md, 'o default que nunca foi uma decisao')"
    );
    assert!(
        !script.contains("[string]$Model = \"deepseek"),
        "o provider `deepseek/` nao esta autenticado na maquina de operacao desde a migracao \
         para Linux (`opencode auth` so tem `opencode-go`): com ele no default a rodada morre no \
         provedor, nao no codigo, e a metrica registra falha sem causa"
    );
}

#[test]
fn variante_do_modelo_e_repassada_ao_cli_e_nao_so_declarada() {
    let script = oc_iter_content();

    assert!(
        script.contains("[string]$Variant = \"max\""),
        "o esforco de raciocinio vem da flag `--variant` do `opencode run` e o default tem de ser \
         max. O handoff da 0137 afirmava `reasoningEffort max ja configurado em \
         ~/.config/opencode` e o arquivo tinha so o $schema — a afirmacao nao sobreviveu a troca \
         de maquina. Declarado no script, o esforco viaja com o repo"
    );
    let declaracao = script
        .find("[string]$Variant")
        .expect("parametro conferido acima");
    let uso = script.find("\"--variant\", $Variant").expect(
        "o parametro tem de ser REPASSADO ao CLI (`\"--variant\", $Variant`), nao so declarado: \
         parametro declarado e nunca usado e decoracao que mente — a rodada corre no esforco \
         default do provedor e a metricas.csv credita um esforco que nao foi usado",
    );
    assert!(declaracao < uso, "declaracao antes do uso");
}

#[test]
fn variante_vazia_nao_emite_flag_orfa() {
    let script = oc_iter_content();

    assert!(
        linha_de(&script, "\"--variant\", $Variant").contains("if ($Variant)"),
        "`-Variant \"\"` tem de suprimir a flag inteira. Emitida com valor vazio, o achatamento \
         do -ArgumentList produz `--variant --format json` e o CLI le `--format` como VALOR da \
         variante: a rodada sai com o help impresso e exit 0 — exatamente o modo de falha do \
         smoke 0008b/2 que o rotulo `falha:sem-execucao` existe para pegar"
    );
}
