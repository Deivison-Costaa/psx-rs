use std::path::{Path, PathBuf};

use crate::app::config::Config;

pub const NOME_DA_APP: &str = "psx-rs";
pub const ARQUIVO_DE_CONFIG: &str = "psx-rs.toml";
pub const ARQUIVO_DE_PERFIL: &str = "controles.txt";
pub const SUBPASTA_CARTOES: &str = "cartoes";
pub const SUBPASTA_SAVES: &str = "saves";

/// As variaveis de ambiente que importam, lidas pelo frontend: o `psx-core` nao le
/// ambiente (R3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ambiente {
    pub home: Option<String>,
    pub xdg_config_home: Option<String>,
    pub xdg_data_home: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pastas {
    arquivo: PathBuf,
    config: PathBuf,
    dados: PathBuf,
    casa: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copia {
    pub origem: PathBuf,
    pub destino: PathBuf,
}

fn absoluta(valor: Option<&str>) -> Option<PathBuf> {
    let texto = valor.map(str::trim).filter(|s| !s.is_empty())?;
    let caminho = PathBuf::from(texto);
    caminho.is_absolute().then_some(caminho)
}

fn texto(caminho: &Path) -> String {
    caminho.to_string_lossy().to_string()
}

impl Pastas {
    /// XDG Base Directory: variavel vazia ou relativa e ignorada. Sem HOME nem XDG, tudo
    /// cai na pasta atual, que era o comportamento antigo.
    pub fn padrao(ambiente: &Ambiente, atual: &Path) -> Pastas {
        let casa = absoluta(ambiente.home.as_deref());
        let base = |xdg: Option<&str>, sufixo: &str| {
            absoluta(xdg)
                .or_else(|| casa.as_ref().map(|c| c.join(sufixo)))
                .map(|b| b.join(NOME_DA_APP))
                .unwrap_or_else(|| atual.to_path_buf())
        };
        let config = base(ambiente.xdg_config_home.as_deref(), ".config");
        let dados = base(ambiente.xdg_data_home.as_deref(), ".local/share");
        Pastas {
            arquivo: config.join(ARQUIVO_DE_CONFIG),
            config,
            dados,
            casa,
        }
    }

    pub fn com_config_em(&self, arquivo: &Path, atual: &Path) -> Pastas {
        let arquivo = atual.join(arquivo);
        let config = arquivo
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| atual.to_path_buf());
        Pastas {
            arquivo,
            config,
            ..self.clone()
        }
    }

    pub fn arquivo_de_config(&self) -> PathBuf {
        self.arquivo.clone()
    }

    pub fn pasta_de_config(&self) -> &Path {
        &self.config
    }

    pub fn perfil_de_controle(&self) -> PathBuf {
        self.config.join(ARQUIVO_DE_PERFIL)
    }

    pub fn cartoes_padrao(&self) -> PathBuf {
        self.dados.join(SUBPASTA_CARTOES)
    }

    pub fn saves_padrao(&self) -> PathBuf {
        self.dados.join(SUBPASTA_SAVES)
    }

    pub fn casa(&self) -> Option<&Path> {
        self.casa.as_deref()
    }

    pub fn resolve(&self, valor: &str) -> PathBuf {
        if let (Some(resto), Some(casa)) = (valor.strip_prefix("~/"), &self.casa) {
            return casa.join(resto);
        }
        self.config.join(valor)
    }

    fn pasta_ou(&self, valor: &str, padrao: PathBuf) -> String {
        if valor.trim().is_empty() {
            texto(&padrao)
        } else {
            texto(&self.resolve(valor))
        }
    }

    pub fn efetiva(&self, config: &Config) -> Config {
        let jogos_padrao = self.casa.clone().unwrap_or_else(|| self.config.clone());
        Config {
            bios: if config.bios.trim().is_empty() {
                String::new()
            } else {
                texto(&self.resolve(&config.bios))
            },
            pasta_de_jogos: self.pasta_ou(&config.pasta_de_jogos, jogos_padrao),
            pasta_de_cartoes: self.pasta_ou(&config.pasta_de_cartoes, self.cartoes_padrao()),
            pasta_de_saves: self.pasta_ou(&config.pasta_de_saves, self.saves_padrao()),
            ..config.clone()
        }
    }
}

/// O que copiar da pasta atual (layout antigo) para as pastas novas. So entra o que
/// existe na origem e falta no destino: nada que ja esta no lugar novo e sobrescrito.
pub fn migracoes<F>(pastas: &Pastas, atual: &Path, existe: F) -> Vec<Copia>
where
    F: Fn(&Path) -> bool,
{
    let candidatos = [
        (atual.join(ARQUIVO_DE_CONFIG), pastas.arquivo_de_config()),
        (atual.join(ARQUIVO_DE_PERFIL), pastas.perfil_de_controle()),
        (atual.join(SUBPASTA_CARTOES), pastas.cartoes_padrao()),
        (atual.join(SUBPASTA_SAVES), pastas.saves_padrao()),
    ];
    candidatos
        .into_iter()
        .filter(|(origem, destino)| origem != destino && existe(origem) && !existe(destino))
        .map(|(origem, destino)| Copia { origem, destino })
        .collect()
}

fn ancorado(valor: &str, origem: &Path) -> String {
    if valor == "." {
        return texto(origem);
    }
    if valor.trim().is_empty() || Path::new(valor).is_absolute() || valor.starts_with("~/") {
        valor.to_string()
    } else {
        texto(&origem.join(valor))
    }
}

/// Relativos da config antiga eram da pasta atual; `cartoes`/`saves` padrao viram vazio
/// porque o conteudo deles foi copiado para a pasta de dados.
pub fn config_migrada(config: &Config, origem: &Path) -> Config {
    let dado = |valor: &str, legado: &str| {
        if valor == legado {
            String::new()
        } else {
            ancorado(valor, origem)
        }
    };
    Config {
        bios: ancorado(&config.bios, origem),
        pasta_de_jogos: ancorado(&config.pasta_de_jogos, origem),
        pasta_de_cartoes: dado(&config.pasta_de_cartoes, SUBPASTA_CARTOES),
        pasta_de_saves: dado(&config.pasta_de_saves, SUBPASTA_SAVES),
        ..config.clone()
    }
}
