# Fighting Stick MIDI

Aplicativo Windows que transforma um arcade stick reconhecido pelo SDL2 em notas MIDI para o Ableton Live Lite, usando uma porta virtual loopMIDI. Não requer Max for Live.

## Começar

### Download pronto para Windows

Baixe `FightingStickMidi.exe` na página de [Releases](https://github.com/anilsonlopes/fighting-stick/releases) e execute-o. O loopMIDI continua sendo necessário.

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
git tag -a v0.2.0 -m "Fighting Stick MIDI v0.2.0"
git push origin v0.2.0
```

Cada Release contém somente o executável Windows. A documentação fica acessível pelo menu **Ajuda** do aplicativo e pelo repositório. A pasta local `target/` permanece fora do Git.
