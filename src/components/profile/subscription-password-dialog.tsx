import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
} from "@mui/material";
import { useState } from "react";
import { useTranslation } from "react-i18next";

const SUBSCRIPTION_NEED_PASSWORD = "SUBSCRIPTION_NEED_PASSWORD";
const SUBSCRIPTION_WRONG_PASSWORD = "SUBSCRIPTION_WRONG_PASSWORD";

export function isSubscriptionNeedPassword(err: unknown): boolean {
  const msg = typeof err === "string" ? err : (err as Error)?.message ?? "";
  return msg.includes(SUBSCRIPTION_NEED_PASSWORD);
}

export function isSubscriptionWrongPassword(err: unknown): boolean {
  const msg = typeof err === "string" ? err : (err as Error)?.message ?? "";
  return msg.includes(SUBSCRIPTION_WRONG_PASSWORD);
}

export function isSubscriptionPasswordError(err: unknown): boolean {
  return isSubscriptionNeedPassword(err) || isSubscriptionWrongPassword(err);
}

interface SubscriptionPasswordDialogProps {
  open: boolean;
  wrongPassword?: boolean;
  initialValue?: string;
  onConfirm: (password: string) => void;
  onCancel: () => void;
}

export function SubscriptionPasswordDialog({
  open,
  wrongPassword = false,
  initialValue = "",
  onConfirm,
  onCancel,
}: SubscriptionPasswordDialogProps) {
  const { t } = useTranslation();
  const [password, setPassword] = useState(initialValue);

  const handleConfirm = () => {
    onConfirm(password);
    setPassword("");
  };

  const handleCancel = () => {
    setPassword("");
    onCancel();
  };

  return (
    <Dialog open={open} onClose={handleCancel} maxWidth="xs" fullWidth>
      <DialogTitle>
        {t("profiles.modals.profileForm.fields.loginPasswordDialogTitle")}
      </DialogTitle>
      <DialogContent>
        <TextField
          sx={{ mt: 1 }}
          autoFocus
          fullWidth
          size="small"
          type="password"
          label={t("profiles.modals.profileForm.fields.loginPassword")}
          placeholder={t(
            "profiles.modals.profileForm.fields.loginPasswordPlaceholder",
          )}
          value={password}
          error={wrongPassword}
          helperText={
            wrongPassword
              ? t("profiles.modals.profileForm.fields.loginPasswordWrongHint")
              : undefined
          }
          onKeyDown={(e) => {
            if (e.key === "Enter") handleConfirm();
          }}
          onChange={(e) => setPassword(e.target.value)}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={handleCancel}>{t("shared.actions.cancel")}</Button>
        <Button variant="contained" onClick={handleConfirm}>
          {t("shared.actions.confirm")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
