export type TourPlacement = "top" | "bottom" | "left" | "right";

export interface TourStep {
  id: string;
  titleKey: string;
  descKey: string;
  placement: TourPlacement;
}

export const tourSteps: TourStep[] = [
  { id: "tour-home", titleKey: "tour.home.title", descKey: "tour.home.desc", placement: "right" },
  { id: "tour-events", titleKey: "tour.events.title", descKey: "tour.events.desc", placement: "right" },
  { id: "tour-project-build", titleKey: "tour.projectBuild.title", descKey: "tour.projectBuild.desc", placement: "bottom" },
  { id: "tour-project-deploy", titleKey: "tour.projectDeploy.title", descKey: "tour.projectDeploy.desc", placement: "bottom" },
  { id: "tour-project-settings", titleKey: "tour.projectSettings.title", descKey: "tour.projectSettings.desc", placement: "left" },
  { id: "tour-settings", titleKey: "tour.settings.title", descKey: "tour.settings.desc", placement: "right" },
  { id: "tour-switch-workspace", titleKey: "tour.switchWorkspace.title", descKey: "tour.switchWorkspace.desc", placement: "right" },
];
