import icon from "../../../assets/icon.png";

export default function Logo({ size = 32 }: { size?: number }) {
  return <img src={icon} width={size} height={size} alt="" className="shrink-0 rounded-lg" />;
}
