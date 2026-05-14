pub fn translate(key: &str) -> &'static str {
    match key {
        "patto_project_root_missing" => "No se encontró la raiz del proyecto.",
        "patto_project_root_missing.hint" => {
            "Ejecuta este comando dentro de un proyecto Patto o usa --root."
        }
        "patto_root_not_directory" => "La ruta informada no es un directorio",
        "patto_root_not_directory.hint" => "Informa la ruta raíz del directorio",
        "patto_config_missing" => "No se encontró .patto/config.json.",
        "patto_config_missing.hint" => {
            "Crea .patto/config.json con { \"schemaVersion\": 1, \"lang\": \"es\" }."
        }
        "patto_config_invalid" => "No se pudo leer .patto/config.json como JSON válido.",
        "patto_config_invalid.hint" => "Revisa la sintaxis JSON del archivo de configuración.",
        "patto_config_lang_unsupported" => "El idioma configurado todavía no está soportado.",
        "patto_config_lang_unsupported.hint" => "Por ahora usa \"lang\": \"es\".",
        "patto_package_json_missing" => "No se encontró package.json en la raíz del proyecto.",
        "patto_package_json_missing.hint" => {
            "Ejecuta scan desde la raíz de un proyecto Patto o usa --root."
        }
        "patto_commands_dir_missing" => "No se encontró src/commands.",
        "patto_commands_dir_missing.hint" => {
            "Un proyecto Patto debe tener sus comandos en src/commands."
        }
        "patto_scan_index_write_failed" => "No se pudo escribir .patto/index.json.",
        "patto_scan_index_write_failed.hint" => {
            "Revisa permisos de escritura sobre la carpeta .patto."
        }
        "patto_source_file_read_failed.message" => "No se pudo leer el archivo `{file}`.",
        "patto_source_file_read_failed" => "No se pudo leer un archivo fuente del proyecto.",
        "patto_source_file_read_failed.hint" => {
            "Revisa permisos, codificación y que el archivo siga existiendo durante el análisis."
        }
        "common.unknown-class" => "(clase desconocida)",
        "common.unnamed" => "(sin nombre)",
        "common.unknown-version" => "(sin versión)",
        "common.not-written" => "(no escrito)",
        "cli.scan.root-invalid" => "patto-core scan failed: la raíz del proyecto no es válida.",
        "cli.scan.completed" => {
            "patto-core scan completed: {files} archivos, {commands} comandos, índice en {index}."
        }
        "cli.lint.root-invalid" => "patto-core lint failed: la raíz del proyecto no es válida.",
        "cli.lint.completed" => {
            "patto-core lint completed: {errors} errores, {warnings} advertencias, {infos} infos."
        }
        "cli.check.root-invalid" => "patto-core check failed: la raíz del proyecto no es válida.",
        "cli.check.completed" => {
            "patto-core check completed: {errors} errores, {warnings} advertencias, {infos} infos."
        }
        "cli.doctor.root-invalid" => "patto-core doctor failed: la raíz del proyecto no es válida.",
        "cli.doctor.completed" => {
            "patto-core doctor completed: {ok} ok, {warnings} advertencias, {errors} errores, {skipped} omitidos."
        }
        "cli.format-i18n.root-invalid" => {
            "patto-core format-i18n failed: la raíz del proyecto no es válida."
        }
        "cli.format-i18n.completed" => {
            "patto-core format-i18n completed: {formatted} formateados, {unchanged} sin cambios, {skipped} omitidos."
        }
        "patto_format_i18n_locale_dir_missing" => "No se encontró src/i18n/locale.",
        "patto_format_i18n_locale_dir_missing.hint" => {
            "Crea los archivos de locale antes de ejecutar format-i18n."
        }
        "patto_format_i18n_file_unsupported.message" => {
            "No se pudo formatear `{file}` porque no tiene un export const de locale soportado."
        }
        "patto_format_i18n_file_unsupported.hint" => {
            "Usa archivos con forma export const es = { 'key': 'value' }."
        }
        "patto_format_i18n_file_read_failed.message" => "No se pudo leer `{file}`.",
        "patto_format_i18n_file_read_failed.hint" => {
            "Revisa permisos y que el archivo siga existiendo."
        }
        "patto_format_i18n_file_write_failed.message" => "No se pudo escribir `{file}`.",
        "patto_format_i18n_file_write_failed.hint" => {
            "Revisa permisos de escritura en src/i18n/locale."
        }
        "patto_lint_rule_config_invalid.message" => {
            "La regla `{rule}` tiene severidad inválida: `{severity}`."
        }
        "patto_lint_rule_config_invalid" => {
            "La configuración de lint-rules contiene una severidad inválida."
        }
        "patto_lint_rule_config_invalid.hint" => {
            "Usa una de estas severidades: off, info, warning o error."
        }
        "duplicate-commands.message" => "La key de comando `{key}` está duplicada.",
        "duplicate-commands" => "Hay comandos con la misma clave.",
        "duplicate-commands.hint" => "Renombra uno de los comandos para que su key sea única.",
        "duplicate-aliases.duplicate.message" => "El alias `{alias}` está duplicado.",
        "duplicate-aliases.name-conflict.message" => {
            "El alias `{alias}` entra en conflicto con un nombre de comando."
        }
        "duplicate-aliases" => "Hay aliases de comando duplicados.",
        "duplicate-aliases.hint" => "Cada alias debe apuntar a un solo comando base.",
        "unknown-command-files.message" => "No se detectó metadata de comando en `{file}`.",
        "unknown-command-files" => "Hay archivos *.command.ts sin metadata de comando detectable.",
        "unknown-command-files.hint" => {
            "Agrega @Command, @Subcommand o @SubcommandGroup, o extiende una definition decorada."
        }
        "invalid-command-names.message" => "El nombre `{value}` no es compatible con Discord.",
        "invalid-command-names" => "Hay nombres de comando incompatibles con Discord.",
        "invalid-command-names.hint" => {
            "Usa 1-32 caracteres en lowercase, números, guiones o underscores."
        }
        "decorated-base-command.message" => {
            "La clase `{class}` está decorada pero no extiende BaseCommand."
        }
        "decorated-base-command" => "Hay clases decoradas que no extienden BaseCommand.",
        "decorated-base-command.hint" => {
            "Extiende BaseCommand directamente o mediante una definition que lo extienda."
        }
        "missing-run-method.message" => "El comando en `{file}` no implementa run().",
        "missing-run-method" => "Hay comandos sin método run().",
        "missing-run-method.hint" => "Implementa async run(): Promise<void> en el comando.",
        "subcommand-consistency.message" => {
            "El comando en `{file}` tiene subcomandos o grupos inconsistentes."
        }
        "subcommand-consistency" => "Hay subcomandos o grupos inconsistentes.",
        "subcommand-consistency.hint" => {
            "Revisa parent, name y subcommand en @Subcommand o @SubcommandGroup."
        }
        "ghost-parent-mix.message" => {
            "El comando padre `{name}` está declarado como @Command y también agrupa subcomandos."
        }
        "ghost-parent-mix" => "Un comando padre existe como @Command y también agrupa subcomandos.",
        "ghost-parent-mix.hint" => {
            "Deja que el padre sea fantasma o evita usar el mismo nombre para subcomandos."
        }
        "invalid-arguments.duplicate.message" => "El argumento `{name}` está duplicado.",
        "invalid-arguments.required-order.message" => {
            "El argumento requerido `{name}` aparece después de uno opcional."
        }
        "invalid-arguments.raw-text-position.message" => {
            "El argumento `{name}` con rawText debe ser el último."
        }
        "invalid-arguments.choice-type.message" => {
            "La opción `{choice}` no coincide con el tipo `{type}`."
        }
        "invalid-arguments" => "Hay argumentos @Arg inconsistentes.",
        "invalid-arguments.hint" => {
            "Evita nombres duplicados, required después de optional, rawText antes de otros args y choices incompatibles."
        }
        "command-folder-convention.message" => {
            "El comando en `{file}` no cumple la convención de carpetas."
        }
        "command-folder-convention" => "Hay comandos fuera de la convención de carpetas.",
        "command-folder-convention.hint" => {
            "Usa src/commands/<categoria>, src/commands/<parent> o src/commands/<parent>/<group> según el tipo."
        }
        "broken-alias-imports.message" => "No se pudo resolver el import `{import}`.",
        "broken-alias-imports" => "Hay imports con alias @/ que no se pueden resolver.",
        "broken-alias-imports.hint" => "Asegúrate de que @/ apunte a src/ y que el archivo exista.",
        "plugin-specified-commands.message" => {
            "PluginScope.Specified no tiene una lista de commands válida."
        }
        "plugin-specified-commands" => "Hay PluginScope.Specified sin lista de commands válida.",
        "plugin-specified-commands.hint" => {
            "Agrega commands: [MiCommand] cuando uses PluginScope.Specified."
        }
        "sharding-redis-config.redis-url.message" => "REDIS_URL no está definido o está vacío.",
        "sharding-redis-config.missing-store.message" => "Falta el store Redis `{file}`.",
        "sharding-redis-config" => "Sharding está habilitado sin configuración Redis completa.",
        "sharding-redis-config.hint" => {
            "Define REDIS_URL y conserva los stores Redis configurables del template."
        }
        "component-handler-methods.message" => {
            "No existe el método estático `{method}` para el componente."
        }
        "component-handler-methods" => {
            "Hay componentes que apuntan a métodos estáticos inexistentes."
        }
        "component-handler-methods.hint" => {
            "Agrega el método static correspondiente en la clase del comando."
        }
        "feature-config.invalid-shape.message" => {
            "`features` debe ser un objeto de flags booleanos."
        }
        "feature-config.unknown.message" => {
            "La feature `{feature}` no está soportada por este core."
        }
        "feature-config.non-boolean.message" => {
            "La feature `{feature}` debe configurarse como boolean."
        }
        "feature-config" => "La configuración de features no coincide con el schema soportado.",
        "feature-config.hint" => {
            "Usa features conocidas, por ejemplo: { \"features\": { \"i18n\": true } }."
        }
        "i18n-missing-keys.message" => "La key i18n `{key}` no existe en `{file}`.",
        "i18n-missing-keys" => "Hay keys i18n usadas en código sin traducción.",
        "i18n-missing-keys.hint" => {
            "Agrega la key en todos los archivos existentes de src/i18n/locale o cambia la llamada."
        }
        "i18n-dynamic-keys.message" => {
            "Esta key i18n es dinámica y no puede validarse estáticamente."
        }
        "i18n-dynamic-keys" => "Hay llamadas i18n con keys dinámicas.",
        "i18n-dynamic-keys.hint" => {
            "Prefiere this.t('clave.literal') para que el linter pueda validar las traducciones."
        }
        "i18n-locale-parity.message" => "La key i18n `{key}` falta en el locale `{locale}`.",
        "i18n-locale-parity" => "Los archivos de locale no tienen el mismo conjunto de keys.",
        "i18n-locale-parity.hint" => {
            "Mantén src/i18n/locale/es.ts, en.ts y pt.ts con las mismas keys."
        }
        "doctor-env.template-missing.message" => "No se encontró .env.template.",
        "doctor-env.env-missing.message" => "No se encontró .env.",
        "doctor-env.required-missing.message" => "Falta la variable `{name}` en .env.",
        "doctor-env.check.title" => "Variables de entorno",
        "doctor-env.detail.template-present" => ".env.template presente",
        "doctor-env.detail.env-present" => ".env presente",
        "doctor-env.detail.message-content" => {
            "USE_MESSAGE_CONTENT=true requiere activar Message Content Intent en Discord Developer Portal"
        }
        "doctor-runtime.node-version.message" => "Node.js {version} es menor a 18.",
        "doctor-runtime.node-missing.message" => {
            "Node.js no está instalado o no está disponible en PATH."
        }
        "doctor-runtime.package-manager-missing.message" => "No se encontró `{command}` en PATH.",
        "doctor-runtime.check.title" => "Node.js y gestor de paquetes",
        "doctor-runtime.detail.command-version" => "{command}: {version}",
        "doctor-runtime.detail.package-manager-unknown" => "package manager: desconocido",
        "doctor-package-json.missing.message" => "No se encontró package.json.",
        "doctor-package-json.missing-dependency.message" => "Falta la dependencia `{dependency}`.",
        "doctor-package-json.missing-script.message" => "Falta el script `{script}`.",
        "doctor-package-json.check.title" => "package.json",
        "doctor-package-json.detail.package" => "package: {name}@{version}",
        "doctor-package-json.detail.scripts" => "scripts: {scripts}",
        "doctor-tsconfig.missing.message" => "No se encontró tsconfig.json.",
        "doctor-tsconfig.option-required.message" => {
            "La opción `{key}` debe estar habilitada en tsconfig.json."
        }
        "doctor-tsconfig.alias-missing.message" => "Falta el alias @/* en tsconfig.json.",
        "doctor-tsconfig.check.title" => "Configuración TypeScript",
        "doctor-tsconfig.detail.read" => "tsconfig.json leído",
        "doctor-project-config.missing.message" => "No se encontró `{path}`.",
        "doctor-project-config.invalid-json.message" => {
            "No se pudo leer `{path}` como JSON válido."
        }
        "doctor-project-config.unsupported-lang.message" => {
            "El idioma `{lang}` todavía no está soportado."
        }
        "doctor-project-config.check.title" => ".patto/config.json",
        "doctor-project-config.detail.lang" => "lang: {lang}",
        "doctor-sharding-redis.redis-url.message" => "REDIS_URL no está definido o está vacío.",
        "doctor-sharding-redis.missing-store.message" => "Falta el store Redis `{file}`.",
        "doctor-sharding-redis.check.title" => "Sharding/Redis",
        "doctor-sharding-redis.detail.disabled" => "SHARDING_ENABLED no está activo",
        "doctor-sharding-redis.detail.enabled" => "SHARDING_ENABLED=true",
        "doctor-sharding-redis.detail.store-present" => "{file}: presente",
        "doctor-build-output.missing.message" => "No se encontró la salida de build en `{main}`.",
        "doctor-build-output.check.title" => "Salida de build",
        "doctor-build-output.detail.main" => "main: {main}",
        "doctor-build-output.detail.main-missing" => "package.json no define main",
        "doctor-i18n.missing-file.message" => "La feature i18n está activa pero falta `{file}`.",
        "doctor-i18n.base-command-helper.message" => {
            "BaseCommand no expone el helper i18n `t` esperado."
        }
        "doctor-i18n.check.title" => "i18n",
        "doctor-i18n.detail.disabled" => "features.i18n no está activo",
        "doctor-i18n.detail.file-present" => "{file}: presente",
        "doctor-i18n.detail.file-missing" => "{file}: faltante",
        "doctor-runtime" => "El entorno local no cumple los requisitos mínimos.",
        "doctor-runtime.hint" => "Instala Node.js 18 o superior y el package manager del proyecto.",
        "doctor-package-json" => "package.json no contiene la configuración esperada.",
        "doctor-package-json.hint" => "Revisa scripts y dependencias base de Patto Bot Template.",
        "doctor-env" => "La configuración de entorno está incompleta.",
        "doctor-env.hint" => "Copia .env.template a .env y completa BOT_TOKEN y CLIENT_ID.",
        "doctor-tsconfig" => "tsconfig.json no contiene opciones necesarias para Patto.",
        "doctor-tsconfig.hint" => {
            "Activa experimentalDecorators, emitDecoratorMetadata y el alias @/*."
        }
        "doctor-project-config" => "La configuración .patto/config.json está incompleta.",
        "doctor-project-config.hint" => "Crea .patto/config.json con lang es.",
        "doctor-sharding-redis" => "Sharding/Redis no está configurado correctamente.",
        "doctor-sharding-redis.hint" => {
            "Si SHARDING_ENABLED=true, define REDIS_URL y conserva los stores Redis."
        }
        "doctor-build-output" => "La salida de build no existe todavía.",
        "doctor-build-output.hint" => "Ejecuta el script build antes de usar start en producción.",
        "doctor-i18n" => "El entorno i18n no está completo.",
        "doctor-i18n.hint" => {
            "Si features.i18n está activo, conserva src/i18n, los locales es/en/pt y BaseCommand.t."
        }
        _ => "",
    }
}
