# Fighting Stick MIDI

Aplicativo Windows que transforma um arcade stick reconhecido pelo SDL2 em notas MIDI para o Ableton Live Lite, usando uma porta virtual loopMIDI. Não requer Max for Live.

## Começar

### Download pronto para Windows

Baixe `FightingStickMidi-Setup-<versão>.exe` na página de [Releases](https://github.com/anilsonlopes/fighting-stick/releases) e execute-o. O instalador não exige administrador, cria um atalho no menu Iniciar e oferece um atalho na área de trabalho, marcado por padrão. O loopMIDI continua sendo necessário e deve ser instalado separadamente.

Para atualizar, execute o instalador da versão nova. Para desinstalar, use **Aplicativos instalados** nas Configurações do Windows. Os perfis em `%APPDATA%\FightingStickMidi` são preservados na desinstalação.

### Compilar o código-fonte

1. Abra o PowerShell na raiz do projeto e execute `Set-ExecutionPolicy -Scope Process Bypass`.
2. Execute `.\setup.ps1`. O script instala Rust/Cargo, CMake e o Visual Studio Build Tools com suporte C++ e roda os testes. A instalação pode pedir elevação de administrador.
3. Instale o [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) e crie uma porta chamada `Fighting Stick MIDI`.
4. Execute `cargo run -p fighting-stick-desktop --release`.
5. Conecte o controle, escolha a porta loopMIDI e salve o perfil.
6. Siga [a configuração do Ableton Live](docs/ABLETON.md).

Para instalar somente Rust/Cargo e CMake, sem o compilador C++, use `.\setup.ps1 -SkipBuildTools`; nesse caso, o projeto provavelmente não compilará até o Build Tools ser instalado. Para apenas instalar as ferramentas sem compilar o projeto, use `.\setup.ps1 -SkipProjectCheck`.

Os perfis são gravados em `%APPDATA%\FightingStickMidi`. O padrão envia C1–B1 (notas MIDI 36–47), com velocidade 100.

## Desenvolvimento

- `cargo test --workspace`: testes automatizados.
- `cargo fmt --all -- --check`: formatação.
- `cargo clippy --workspace --all-targets -- -D warnings`: análise estática.

Consulte [arquitetura e decisões](docs/ARCHITECTURE.md) e [solução de problemas](docs/TROUBLESHOOTING.md).

## Publicar uma versão

O workflow `.github/workflows/release.yml` compila e publica automaticamente ao receber uma tag `v*`:

```powershell
git tag -a v0.1.3 -m "Fighting Stick MIDI v0.1.3"
git push origin v0.1.3
```

Antes de criar a tag, ajuste a versão em `Cargo.toml` para o mesmo número (sem o prefixo `v`). Cada Release contém somente o instalador Windows. A documentação fica acessível pelo menu **Ajuda** do aplicativo e pelo repositório. As pastas locais `target/` e `dist/` permanecem fora do Git.
