import { useState } from "react";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  FormField,
  Input,
} from "@gio/bigsu-ui";

interface LoginScreenProps {
  defaultUsername?: string;
  initialError?: string;
  onLogin: (username: string, password: string, remember: boolean) => Promise<void>;
}

function LoginScreen({ defaultUsername = "", initialError = "", onLogin }: LoginScreenProps) {
  const [username, setUsername] = useState(defaultUsername);
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(true);
  const [error, setError] = useState(initialError);
  const [submitting, setSubmitting] = useState(false);

  const submit = async () => {
    if (submitting) return;
    setError("");
    setSubmitting(true);
    try {
      await onLogin(username.trim(), password, remember);
    } catch (err) {
      setError(typeof err === "string" ? err : "Sign-in failed.");
      setPassword("");
    } finally {
      setSubmitting(false);
    }
  };

  const canSubmit = !submitting && username.trim().length > 0 && password.length > 0;

  return (
    <div
      className="flex h-screen items-center justify-center bg-app p-4"
      data-tauri-drag-region
    >
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle>Sign in</CardTitle>
          <CardDescription>Use your corporate LDAP account.</CardDescription>
        </CardHeader>
        <CardContent>
          <form
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              void submit();
            }}
            noValidate
          >
            <FormField label="Username" htmlFor="ldap-username">
              <Input
                id="ldap-username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                autoFocus
                autoCapitalize="off"
                autoCorrect="off"
                spellCheck={false}
                placeholder="e.g. rendi"
              />
            </FormField>
            <FormField
              label="Password"
              htmlFor="ldap-password"
              errorText={error || undefined}
            >
              <Input
                id="ldap-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </FormField>
            <Checkbox
              label="Remember me on this device"
              checked={remember}
              onCheckedChange={(checked) => setRemember(checked === true)}
            />
            <Button type="submit" variant="primary" loading={submitting} disabled={!canSubmit}>
              {submitting ? "Signing in…" : "Sign in"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default LoginScreen;
