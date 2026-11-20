import { open } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../stores/app_store";
// UI: 작업 폴더 입력.
export function DirectoryField({
  label,
  value,
  onChange,
  required = false,
}: {
  label: string;
  value: string;
  onChange: (path: string) => void;
  required?: boolean;
}) {
  // I/O: 폴더 선택.
  async function choose() {
    try {
      const path = await open({ directory: true, multiple: false });
      if (typeof path === "string") onChange(path);
    } catch (error) {
      useAppStore.getState().setError(String(error));
    }
  }
  return (
    <label>
      {label}
      <div className="row">
        <input
          required={required}
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
        <button type="button" onClick={() => void choose()}>
          폴더 선택
        </button>
      </div>
    </label>
  );
}
